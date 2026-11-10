//! Offline, rule-based edits applied to the final transcript:
//! - spoken corrections: "scratch that" drops the sentence before it, and
//!   "no I mean" / "sorry I mean" replace the few words just before the cue;
//! - snippets: a saved trigger phrase is replaced by its saved text.

use crate::settings::Snippet;

/// Most words a "no I mean" correction may take back.
const MAX_CORRECTED_WORDS: usize = 6;

const REPLACE_CUES: [&str; 3] = ["no i mean", "sorry i mean", "no wait i mean"];
/// Cues that start a restatement: "I want strawberries, no, I want bananas".
/// Longest first, so "no wait" wins over "no".
const RESTATE_CUES: [&str; 6] = ["no wait", "or rather", "i mean", "actually", "sorry", "no"];
/// Words after the cue that must repeat earlier words before we drop the
/// earlier version (2+ avoids false hits like "no, I don't think so").
const MIN_RESTATED_WORDS: usize = 2;
const MAX_RESTATED_WORDS: usize = 4;
/// How far back a restatement may reach.
const RESTATE_WINDOW: usize = 25;

const SCRATCH_CUES: [&str; 2] = ["scratch that", "delete that"];

/// Corrections are always on (no setting): free rules first, then the
/// on-device model for changes of mind the rules cannot see, then spoken
/// punctuation and numbers, then snippets.
pub fn apply(text: &str, _corrections: bool, snippets: &[Snippet]) -> String {
    let ruled = apply_corrections(text);
    let fixed = crate::smart_fix::apply(&ruled);
    let corrected = crate::spoken_format::apply(&fixed);
    expand_snippets(&corrected, snippets)
}

/// Lowercase letters/digits only, for matching cue words.
fn norm(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric() || *c == '\'')
        .flat_map(char::to_lowercase)
        .collect()
}

fn ends_clause(word: &str) -> bool {
    word.ends_with(['.', ',', ';', '!', '?', ':'])
}

fn ends_sentence(word: &str) -> bool {
    word.ends_with(['.', '!', '?'])
}

/// Finds a cue (sequence of words) starting at `i`; returns its length.
fn cue_at(words: &[&str], i: usize, cues: &[&str]) -> Option<usize> {
    cues.iter().find_map(|cue| {
        let parts: Vec<&str> = cue.split(' ').collect();
        let fits = i + parts.len() <= words.len()
            && parts.iter().enumerate().all(|(k, p)| norm(words[i + k]) == *p);
        fits.then_some(parts.len())
    })
}

/// If a restatement cue sits at `i` and the words after it repeat earlier
/// words, returns (where to cut `out`, words to skip for the cue).
fn restatement_cut(words: &[&str], i: usize, out: &[String]) -> Option<(usize, usize)> {
    let n = cue_at(words, i, &RESTATE_CUES)?;
    let after = &words[i + n..];
    let from = out.len().saturating_sub(RESTATE_WINDOW);
    for k in (MIN_RESTATED_WORDS..=MAX_RESTATED_WORDS.min(after.len())).rev() {
        let target: Vec<String> = after[..k].iter().map(|w| norm(w)).collect();
        let hit = (from..out.len().saturating_sub(k - 1))
            .rev()
            .find(|&p| (0..k).all(|j| norm(&out[p + j]) == target[j]));
        if let Some(p) = hit {
            return Some((p, n));
        }
    }
    None
}

pub fn apply_corrections(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out: Vec<String> = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        if let Some(n) = cue_at(&words, i, &SCRATCH_CUES) {
            // Drop back to the end of the previous sentence.
            while let Some(last) = out.last() {
                if ends_sentence(last) {
                    break;
                }
                out.pop();
            }
            i += n;
            continue;
        }
        if let Some(cut) = restatement_cut(&words, i, &out) {
            out.truncate(cut.0);
            i += cut.1;
            continue;
        }
        if let Some(n) = cue_at(&words, i, &REPLACE_CUES) {
            // Take back the word just before the cue, or the whole run of
            // capitalised words when it is a name ("New York no I mean Boston").
            let is_cap = |w: &String| w.chars().next().is_some_and(char::is_uppercase);
            let mut taken = 0;
            if let Some(last) = out.pop() {
                taken += 1;
                if is_cap(&last) {
                    while taken < MAX_CORRECTED_WORDS
                        && out.last().is_some_and(|w| is_cap(w) && !ends_clause(w))
                        && out.len() > 1
                    {
                        out.pop();
                        taken += 1;
                    }
                }
            }
            if let Some(last) = out.last_mut() {
                // Keep sentence punctuation but drop a dangling comma.
                if last.ends_with(',') {
                    last.pop();
                }
            }
            i += n;
            continue;
        }
        out.push(words[i].to_string());
        i += 1;
    }
    let joined = out.join(" ");
    // Cues often leave a stray comma or a lowercase start behind.
    let trimmed = joined.trim_start_matches([',', ' ']).trim().to_string();
    capitalize_first(&trimmed, text)
}

fn capitalize_first(result: &str, original: &str) -> String {
    let starts_upper = original.chars().next().is_some_and(char::is_uppercase);
    let mut chars = result.chars();
    match chars.next() {
        Some(c) if starts_upper => c.to_uppercase().chain(chars).collect(),
        Some(c) => std::iter::once(c).chain(chars).collect(),
        None => String::new(),
    }
}

/// Replaces each trigger phrase (whole words, any case, punctuation ignored)
/// with its saved text.
pub fn expand_snippets(text: &str, snippets: &[Snippet]) -> String {
    let mut words: Vec<String> = text.split_whitespace().map(str::to_string).collect();
    for snippet in snippets {
        let trigger: Vec<String> = snippet.trigger.split_whitespace().map(norm).collect();
        if trigger.is_empty() || snippet.text.is_empty() {
            continue;
        }
        let mut i = 0;
        while i + trigger.len() <= words.len() {
            let hit = trigger
                .iter()
                .enumerate()
                .all(|(k, t)| norm(&words[i + k]) == *t);
            if hit {
                let last = &words[i + trigger.len() - 1];
                let tail: String = last
                    .chars()
                    .rev()
                    .take_while(|c| matches!(c, '.' | ',' | '!' | '?' | ';' | ':'))
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                let replacement = format!("{}{}", snippet.text, tail);
                words.splice(i..i + trigger.len(), [replacement]);
                i += 1;
            } else {
                i += 1;
            }
        }
    }
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snip(trigger: &str, text: &str) -> Snippet {
        Snippet {
            trigger: trigger.into(),
            text: text.into(),
        }
    }

    #[test]
    fn no_i_mean_replaces_the_words_before() {
        assert_eq!(
            apply_corrections("Meet me at five, no I mean six o'clock."),
            "Meet me at six o'clock."
        );
        assert_eq!(
            apply_corrections("Send it to Tom no I mean Sarah"),
            "Send it to Sarah"
        );
        assert_eq!(
            apply_corrections("Fly to New York no I mean Boston"),
            "Fly to Boston"
        );
    }

    #[test]
    fn restatement_keeps_only_the_correction() {
        assert_eq!(
            apply_corrections("I want strawberries, no, I want bananas."),
            "I want bananas."
        );
        assert_eq!(
            apply_corrections("Book it for Monday actually book it for Tuesday"),
            "Book it for Tuesday"
        );
        // A "no" that is not a restatement stays.
        let t = "I said no, I don't think so.";
        assert_eq!(apply_corrections(t), t);
    }

    #[test]
    fn scratch_that_drops_the_sentence() {
        assert_eq!(
            apply_corrections("Hello there. This is wrong scratch that. Bye."),
            "Hello there. Bye."
        );
        assert_eq!(apply_corrections("All wrong, scratch that"), "");
    }

    #[test]
    fn spoken_examples_end_to_end() {
        // Real examples, as the speech model writes them.
        assert_eq!(apply("I want strawberries, comma, no, I want bananas.", true, &[]), "I want bananas.");
        assert_eq!(apply("I want strawberries comma no I want bananas", true, &[]), "I want bananas");
        assert_eq!(
            apply("Send it to me comma then call Sam full stop", true, &[]),
            "Send it to me, then call Sam."
        );
        assert_eq!(apply("Use the slash prompt skill", true, &[]), "Use the /prompt skill");
        assert_eq!(apply("I need twenty five chairs", true, &[]), "I need 25 chairs");
    }

    #[test]
    fn plain_text_is_untouched() {
        let t = "I mean it when I say no to this.";
        assert_eq!(apply_corrections(t), t);
        assert_eq!(apply("Normal words here.", true, &[]), "Normal words here.");
    }

    #[test]
    fn snippets_expand_with_punctuation_kept() {
        let s = [snip("my email", "solomon@example.com")];
        assert_eq!(
            expand_snippets("Write to My Email.", &s),
            "Write to solomon@example.com."
        );
        assert_eq!(expand_snippets("nothing here", &s), "nothing here");
    }

    #[test]
    fn empty_snippets_are_ignored() {
        let s = [snip("", "x"), snip("hi", "")];
        assert_eq!(expand_snippets("hi there", &s), "hi there");
    }
}
