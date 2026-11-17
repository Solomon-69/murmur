//! Makes sure "Turn into a prompt" always pastes a prompt in the required
//! Markdown shape (## Goal, then ## Steps as a numbered list).
//!
//! The on-device model usually gets it right, but it can time out, be missing
//! (a build without Apple Intelligence) or drift from the format. A second
//! model call would double the wait, so a drifted answer is repaired here
//! instantly, and with no usable answer the prompt is built from the
//! dictation itself. Murmur never recommends a model: the person picks it.

/// Does the model's answer have the parts every prompt needs?
pub fn has_prompt_format(out: &str) -> bool {
    let Some(goal) = out.find("## Goal") else {
        return false;
    };
    let Some(steps) = out.find("## Steps") else {
        return false;
    };
    goal < steps && out[steps..].lines().skip(1).any(is_numbered_line)
}

fn is_numbered_line(line: &str) -> bool {
    let t = line.trim_start();
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    digits > 0 && t[digits..].starts_with(". ")
}

/// Tidies a valid model answer: no code fences, nothing before "## Goal",
/// no model recommendation, and no sections that only say "None".
pub fn tidy(out: &str) -> String {
    let text = strip_noise(out);
    let start = text.find("## Goal").unwrap_or(0);
    drop_empty_sections(text[start..].trim())
}

/// Removes code fences and any line recommending a model or effort level.
fn strip_noise(out: &str) -> String {
    out.trim()
        .lines()
        .filter(|l| {
            let t = l.trim_start().trim_start_matches(['*', '-', ' ']).to_lowercase();
            !(l.trim_start().starts_with("```")
                || t.starts_with("run with")
                || t.starts_with("recommended model")
                || t.starts_with("model:")
                || t.starts_with("suggested model"))
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Turns a model answer that lost the layout into a prompt that has it,
/// without asking the model again. An answer that looks like the model did
/// the task (code, or far longer than what was said) is not trusted: the
/// prompt is then built from the dictation.
pub fn repair(out: &str, original: &str) -> String {
    let text = strip_noise(out);
    let said = original.split_whitespace().count().max(1);
    let answered = out.contains("```") || text.split_whitespace().count() > said * 2 + 40;
    if text.is_empty() || answered {
        return fallback_prompt(original);
    }
    // Headings are there but the steps are bullets: number them.
    if text.contains("## Goal") && text.contains("## Steps") {
        let renumbered = number_steps(&text);
        if has_prompt_format(&renumbered) {
            return tidy(&renumbered);
        }
    }
    let lines: Vec<String> = text
        .lines()
        .map(strip_markers)
        .filter(|l| !l.is_empty())
        .collect();
    let built = match lines.len() {
        0 => return fallback_prompt(original),
        1 => structure(&lines[0]),
        _ => layout(&lines[0], &lines[1..]),
    };
    if has_prompt_format(&built) {
        built
    } else {
        fallback_prompt(original)
    }
}

/// "- x" / "* x" lines under "## Steps" become "1. x", "2. x".
fn number_steps(text: &str) -> String {
    let mut out = Vec::new();
    let mut in_steps = false;
    let mut n = 0;
    for line in text.lines() {
        if line.starts_with("## ") {
            in_steps = line.trim() == "## Steps";
            out.push(line.to_string());
            continue;
        }
        let t = line.trim_start();
        let item = ["- ", "* ", "• "].iter().find_map(|b| t.strip_prefix(*b));
        match item {
            Some(rest) if in_steps => {
                n += 1;
                out.push(format!("{n}. {}", rest.trim()));
            }
            _ => out.push(line.to_string()),
        }
    }
    out.join("\n")
}

/// A line without its heading, bullet, number or "Goal:" label.
fn strip_markers(line: &str) -> String {
    let mut t = line.trim().trim_start_matches('#').trim();
    for bullet in ["- ", "* ", "• "] {
        if let Some(rest) = t.strip_prefix(bullet) {
            t = rest.trim();
            break;
        }
    }
    let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 && (t[digits..].starts_with(". ") || t[digits..].starts_with(") ")) {
        t = t[digits + 2..].trim();
    }
    for label in ["**Goal:**", "**Goal**:", "**Steps:**", "**Steps**:", "Goal:", "Steps:"] {
        if let Some(rest) = t.strip_prefix(label) {
            t = rest.trim();
            break;
        }
    }
    if matches!(t, "Goal" | "Steps" | "**Goal**" | "**Steps**") {
        return String::new();
    }
    t.to_string()
}

fn layout(goal: &str, steps: &[String]) -> String {
    let goal = sentence_case(goal);
    let numbered = if steps.is_empty() {
        format!("1. {goal}")
    } else {
        steps
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{}. {}", i + 1, sentence_case(s)))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!("## Goal\n{goal}\n\n## Steps\n{numbered}")
}

/// Removes a "## Section" whose whole body is empty or "None" / "N/A".
fn drop_empty_sections(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut section: Vec<String> = Vec::new();
    let flush = |section: &mut Vec<String>, out: &mut Vec<String>| {
        if section.is_empty() {
            return;
        }
        let empty = section.iter().skip(1).all(|l| {
            let t = l
                .trim()
                .trim_start_matches(['-', '*', ' '])
                .trim_end_matches('.')
                .to_lowercase();
            matches!(
                t.as_str(),
                "" | "none"
                    | "n/a"
                    | "na"
                    | "nothing"
                    | "nothing specified"
                    | "none specified"
                    | "not specified"
                    | "none given"
                    | "no questions"
            )
        });
        if !(section[0].starts_with("## ") && empty) {
            out.append(section);
        }
        section.clear();
    };
    for line in text.lines() {
        if line.starts_with("## ") {
            flush(&mut section, &mut out);
        }
        if section.is_empty() && !line.starts_with("## ") {
            out.push(line.to_string());
        } else {
            section.push(line.to_string());
        }
    }
    flush(&mut section, &mut out);
    let joined = out.join("\n");
    // Collapse runs of blank lines left behind.
    let mut result = String::new();
    let mut blank = 0;
    for line in joined.lines() {
        if line.trim().is_empty() {
            blank += 1;
            if blank > 1 {
                continue;
            }
        } else {
            blank = 0;
        }
        result.push_str(line);
        result.push('\n');
    }
    result.trim().to_string()
}

/// Spoken filler that never belongs in a prompt.
const FILLER: &[&str] = &["um", "uh", "erm", "er", "hmm", "uhm"];
/// Openers dropped from the start of a dictation.
const OPENERS: &[&str] = &["okay", "ok", "so", "right", "well", "alright", "and"];

fn clean_words(text: &str) -> String {
    let words: Vec<&str> = text
        .split_whitespace()
        .filter(|w| {
            let bare = w
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            !FILLER.contains(&bare.as_str())
        })
        .collect();
    let mut start = 0;
    while start < words.len() {
        let bare = words[start]
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if OPENERS.contains(&bare.as_str()) {
            start += 1;
        } else {
            break;
        }
    }
    words[start..].join(" ")
}

/// Breaks spoken run-ons where a new ask starts.
static CLAUSE_BREAK: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
    regex::Regex::new(
        r"(?i),?\s+(?:and\s+also|and\s+then|and\s+after\s+that|then|also|after\s+that|plus)\s+|,\s+and\s+|\s+and\s+(?:make\s+sure|please|don't|do\s+not|never)\s+",
    )
    .expect("clause regex")
});

fn sentences(text: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        current.push(c);
        if matches!(c, '.' | '!' | '?') {
            parts.push(std::mem::take(&mut current));
        }
    }
    parts.push(current);
    let parts: Vec<String> = parts
        .into_iter()
        .map(|p| {
            p.trim()
                .trim_end_matches(['.', '!', '?'])
                .trim()
                .to_string()
        })
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() >= 2 {
        return parts;
    }
    // One long spoken sentence: split it where a new ask begins.
    let Some(only) = parts.first() else {
        return Vec::new();
    };
    let mut kept_keyword: Vec<String> = Vec::new();
    let mut last = 0;
    for m in CLAUSE_BREAK.find_iter(only) {
        kept_keyword.push(only[last..m.start()].to_string());
        // Keep "make sure", "don't", "never" as part of the next step.
        let matched = m
            .as_str()
            .trim()
            .trim_start_matches(',')
            .trim()
            .to_lowercase();
        let keep = ["make sure", "please", "don't", "do not", "never"]
            .iter()
            .find(|k| matched.ends_with(*k));
        last = match keep {
            Some(k) => m.end() - k.len() - 1,
            None => m.end(),
        };
    }
    kept_keyword.push(only[last..].to_string());
    kept_keyword
        .into_iter()
        .map(|p| p.trim().trim_matches(',').trim().to_string())
        .filter(|p| p.split_whitespace().count() >= 2)
        .collect()
}

fn sentence_case(s: &str) -> String {
    let s = s.trim();
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => {
            let mut out: String = first.to_uppercase().collect();
            out.push_str(chars.as_str());
            if !out.ends_with(['.', '!', '?']) {
                out.push('.');
            }
            out
        }
        None => String::new(),
    }
}

/// Builds the prompt from the dictation alone: Goal = the first sentence,
/// Steps = the remaining sentences as a numbered list.
pub fn fallback_prompt(text: &str) -> String {
    structure(&clean_words(text))
}

/// Goal and numbered Steps from plain sentences.
fn structure(cleaned: &str) -> String {
    let mut parts = sentences(cleaned);
    if parts.is_empty() {
        parts.push(cleaned.trim().to_string());
    }
    layout(&parts[0], &parts[1..])
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAID: &str = "okay so I want you to go into the murmur app and make the transcription bar a bit wider and also fix the shortcut because it clashes with claude and make sure you test it";

    fn recommends_a_model(out: &str) -> bool {
        let lower = out.to_lowercase();
        ["run with", "sonnet", "opus", "haiku", "recommended model", "effort"]
            .iter()
            .any(|w| lower.contains(w))
    }

    #[test]
    fn detects_the_required_format() {
        let good = "## Goal\nWider bar.\n\n## Steps\n1. Widen the bar.\n2. Test it.";
        assert!(has_prompt_format(good));
        assert!(!has_prompt_format("Make the bar wider and test it."));
        assert!(
            !has_prompt_format("## Goal\nWider bar.\n\n## Steps\n- Widen the bar."),
            "steps must be numbered"
        );
        assert!(
            !has_prompt_format("## Steps\n1. Widen.\n## Goal\nWider."),
            "goal comes first"
        );
    }

    #[test]
    fn fallback_is_always_in_prompt_format_without_a_model_tip() {
        for text in [
            SAID,
            "Fix the login bug. Add a test for it. Do not touch the sign up page.",
            "um build a timer app",
            "",
        ] {
            let out = fallback_prompt(text);
            assert!(out.starts_with("## Goal"), "{out}");
            assert!(has_prompt_format(&out), "{out}");
            assert!(!recommends_a_model(&out), "{out}");
        }
    }

    #[test]
    fn fallback_goal_is_first_sentence_and_steps_the_rest() {
        let out =
            fallback_prompt("Fix the login bug. Add a test for it. Do not touch the sign up page.");
        assert!(out.contains("## Goal\nFix the login bug.\n"), "{out}");
        assert!(
            out.contains("## Steps\n1. Add a test for it.\n2. Do not touch the sign up page."),
            "{out}"
        );
    }

    #[test]
    fn fallback_splits_one_long_spoken_sentence() {
        let out = fallback_prompt(SAID);
        assert!(out.contains("## Goal\nI want you to go into the murmur app and make the transcription bar a bit wider.\n"), "{out}");
        assert!(
            out.contains("1. Fix the shortcut because it clashes with claude."),
            "{out}"
        );
        assert!(out.contains("2. Make sure you test it."), "{out}");
        assert!(!out.to_lowercase().contains("okay so"), "{out}");
    }

    #[test]
    fn tidy_drops_noise_and_any_model_recommendation() {
        let out = tidy("**Run with:** Sonnet · medium effort. Normal job.\n\n## Goal\nX.\n\n## Steps\n1. Do X.\n\n## Questions\n- Nothing specified.");
        assert_eq!(out, "## Goal\nX.\n\n## Steps\n1. Do X.");
        let model = "Here is your prompt:\n```\n**Run with:** Haiku · low effort. Small.\n\n## Goal\nWider bar.\n\n## Steps\n1. Widen it.\n\n## Questions\nNone.\n```";
        assert_eq!(tidy(model), "## Goal\nWider bar.\n\n## Steps\n1. Widen it.");
    }

    #[test]
    fn repair_numbers_bulleted_steps() {
        let out = repair("## Goal\nWider bar.\n\n## Steps\n- Widen the bar.\n- Test it.\n\n## Rules\n- Never touch the menu.", SAID);
        assert_eq!(out, "## Goal\nWider bar.\n\n## Steps\n1. Widen the bar.\n2. Test it.\n\n## Rules\n- Never touch the menu.");
    }

    #[test]
    fn repair_lays_out_an_answer_without_headings() {
        let out = repair("Make the transcription bar wider.\n- Fix the shortcut clash with Claude.\n- Test it.", SAID);
        assert_eq!(out, "## Goal\nMake the transcription bar wider.\n\n## Steps\n1. Fix the shortcut clash with Claude.\n2. Test it.");
        let one = repair("Make the bar wider. Then test it.", SAID);
        assert!(has_prompt_format(&one), "{one}");
        assert!(one.starts_with("## Goal\nMake the bar wider."), "{one}");
    }

    #[test]
    fn repair_never_trusts_an_answer_that_did_the_task() {
        let code = "Sure! Here is the code:\n```rust\nfn main() {}\n```";
        assert_eq!(repair(code, SAID), fallback_prompt(SAID));
        assert_eq!(repair("   ", SAID), fallback_prompt(SAID));
        let essay = "word ".repeat(200);
        assert_eq!(repair(&essay, "make it wider"), fallback_prompt("make it wider"));
    }

    #[test]
    fn repaired_prompts_are_always_in_format() {
        for out in [
            "**Goal:** Wider bar.\n**Steps:**\n1. Widen it.",
            "Goal: Wider bar.",
            "## Goal\nWider bar.",
            "1. Widen it.\n2. Test it.",
        ] {
            let fixed = repair(out, SAID);
            assert!(has_prompt_format(&fixed), "{out} -> {fixed}");
            assert!(!recommends_a_model(&fixed), "{fixed}");
        }
    }
}
