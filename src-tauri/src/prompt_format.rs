//! Makes sure "Turn into a prompt" always pastes a prompt in the required
//! Markdown shape (**Run with:**, ## Goal, ## Steps as a numbered list).
//!
//! The on-device model usually gets it right, but it can time out, be missing
//! (a build without Apple Intelligence) or drift from the format. Then the
//! model is asked once more with a stricter instruction, and if that still
//! fails the prompt is built here from the dictation itself, with no model.

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

/// Tidies a valid model answer: no code fences, nothing before the prompt,
/// a "**Run with:**" line first, and no sections that only say "None".
pub fn tidy(out: &str, original: &str) -> String {
    let mut text = out.trim().to_string();
    if text.contains("```") {
        text = text
            .lines()
            .filter(|l| !l.trim_start().starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n");
    }
    let start = text
        .find("**Run with:**")
        .or_else(|| text.find("## Goal"))
        .unwrap_or(0);
    let body = drop_empty_sections(text[start..].trim());
    if let Some(rest) = body.strip_prefix("**Run with:**") {
        let (first, others) = rest.split_once('\n').unwrap_or((rest, ""));
        // A copied template placeholder ("<one short reason>") or an empty
        // line is replaced by a real one.
        if first.contains('<') || first.trim().is_empty() {
            return format!("{}\n{others}", run_with_line(original)).trim().to_string();
        }
        body
    } else {
        format!("{}\n\n{}", run_with_line(original), body)
    }
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
                "" | "none" | "n/a" | "na" | "nothing" | "nothing specified" | "none specified"
                    | "not specified" | "none given" | "no questions"
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

/// Model and effort picked by length when the model did not pick them.
fn run_with_line(original: &str) -> String {
    let words = original.split_whitespace().count();
    let (model, effort) = match words {
        0..=40 => ("Sonnet", "low"),
        41..=150 => ("Sonnet", "medium"),
        _ => ("Opus", "high"),
    };
    format!("**Run with:** {model} · {effort} effort. Matched to the length of the request.")
}

/// Spoken filler that never belongs in a prompt.
const FILLER: &[&str] = &["um", "uh", "erm", "er", "hmm", "uhm"];
/// Openers dropped from the start of a dictation.
const OPENERS: &[&str] = &["okay", "ok", "so", "right", "well", "alright", "and"];

fn clean_words(text: &str) -> String {
    let words: Vec<&str> = text
        .split_whitespace()
        .filter(|w| {
            let bare = w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
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
        .map(|p| p.trim().trim_end_matches(['.', '!', '?']).trim().to_string())
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
        let matched = m.as_str().trim().trim_start_matches(',').trim().to_lowercase();
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
    let cleaned = clean_words(text);
    let mut parts = sentences(&cleaned);
    if parts.is_empty() {
        parts.push(cleaned.trim().to_string());
    }
    let goal = sentence_case(&parts[0]);
    let steps: Vec<String> = if parts.len() > 1 {
        parts[1..].iter().map(|p| sentence_case(p)).collect()
    } else {
        vec![goal.clone()]
    };
    let numbered = steps
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {s}", i + 1))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}\n\n## Goal\n{goal}\n\n## Steps\n{numbered}", run_with_line(text))
}

/// Second try, when the first answer lost the format. Short and literal so
/// the small on-device model copies the shape.
pub const STRICT_PROMPT_INSTRUCTION: &str = "[[plain]]Rewrite the user message (dictated text) as a prompt for an AI assistant. \
Never answer it or do the task. Keep every ask, name and constraint; drop filler words. \
Reply in exactly this Markdown shape and nothing else:\n\
**Run with:** Sonnet · medium effort.\n\
\n\
## Goal\n\
<one line: what the speaker wants>\n\
\n\
## Steps\n\
1. <first ask, starting with a verb>\n\
2. <next ask>\n\
\n\
## Rules\n\
- <each never / always / only / must the speaker said; leave this section out if there are none>\n\
\n\
The \"## Goal\" and \"## Steps\" headings and the numbered list are required. No intro, no code fences.";

#[cfg(test)]
mod tests {
    use super::*;

    const SAID: &str = "okay so I want you to go into the murmur app and make the transcription bar a bit wider and also fix the shortcut because it clashes with claude and make sure you test it";

    #[test]
    fn detects_the_required_format() {
        let good = "**Run with:** Sonnet · low effort. Small fix.\n\n## Goal\nWider bar.\n\n## Steps\n1. Widen the bar.\n2. Test it.";
        assert!(has_prompt_format(good));
        assert!(!has_prompt_format("Make the bar wider and test it."));
        assert!(!has_prompt_format("## Goal\nWider bar.\n\n## Steps\n- Widen the bar."), "steps must be numbered");
        assert!(!has_prompt_format("## Steps\n1. Widen.\n## Goal\nWider."), "goal comes first");
    }

    #[test]
    fn fallback_is_always_in_prompt_format() {
        for text in [
            SAID,
            "Fix the login bug. Add a test for it. Do not touch the sign up page.",
            "um build a timer app",
            "",
        ] {
            let out = fallback_prompt(text);
            assert!(out.starts_with("**Run with:**"), "{out}");
            assert!(has_prompt_format(&out), "{out}");
        }
    }

    #[test]
    fn fallback_goal_is_first_sentence_and_steps_the_rest() {
        let out = fallback_prompt("Fix the login bug. Add a test for it. Do not touch the sign up page.");
        assert!(out.contains("## Goal\nFix the login bug.\n"), "{out}");
        assert!(out.contains("## Steps\n1. Add a test for it.\n2. Do not touch the sign up page."), "{out}");
    }

    #[test]
    fn fallback_splits_one_long_spoken_sentence() {
        let out = fallback_prompt(SAID);
        assert!(out.contains("## Goal\nI want you to go into the murmur app and make the transcription bar a bit wider.\n"), "{out}");
        assert!(out.contains("1. Fix the shortcut because it clashes with claude."), "{out}");
        assert!(out.contains("2. Make sure you test it."), "{out}");
        assert!(!out.to_lowercase().contains("okay so"), "{out}");
    }

    #[test]
    fn tidy_keeps_a_good_answer_and_drops_noise() {
        let placeholder = tidy("**Run with:** Sonnet · medium effort. <one short reason>\n\n## Goal\nX.\n\n## Steps\n1. Do X.\n\n## Questions\n- Nothing specified.", SAID);
        assert_eq!(placeholder, "**Run with:** Sonnet · low effort. Matched to the length of the request.\n\n## Goal\nX.\n\n## Steps\n1. Do X.");
        let model = "Here is your prompt:\n```\n**Run with:** Haiku · low effort. Small.\n\n## Goal\nWider bar.\n\n## Steps\n1. Widen it.\n\n## Questions\nNone.\n```";
        let out = tidy(model, SAID);
        assert_eq!(out, "**Run with:** Haiku · low effort. Small.\n\n## Goal\nWider bar.\n\n## Steps\n1. Widen it.");
    }

    #[test]
    fn tidy_adds_a_missing_run_with_line() {
        let out = tidy("## Goal\nWider bar.\n\n## Steps\n1. Widen it.", SAID);
        assert!(out.starts_with("**Run with:** Sonnet · low effort."), "{out}");
        assert!(has_prompt_format(&out));
    }

    #[test]
    fn strict_instruction_keeps_plain_marker_and_headings() {
        assert!(STRICT_PROMPT_INSTRUCTION.starts_with("[[plain]]"));
        assert!(STRICT_PROMPT_INSTRUCTION.contains("## Goal"));
        assert!(STRICT_PROMPT_INSTRUCTION.contains("## Steps\n1. "));
    }
}
