//! Smart self-correction: when someone changes their mind mid-sentence
//! ("I like strawberries, or oh, I want bananas") or repeats themselves, the
//! on-device model keeps only what they meant. Always on, no setting.
//!
//! The fixed rules in `voice_edits` run first and are free. This step only
//! runs when the text still shows signs of a correction or a repeat, so
//! ordinary dictation never waits for the model. If the model is missing,
//! slow to start or gives something odd, the text is left exactly as it was.

/// Words that signal a change of mind.
const CUES: &[&str] = &[
    "or oh", "oh no", "no wait", "hold on", "i mean", "or rather", "sorry i meant",
    "scratch that", "never mind", "nevermind", "let me rephrase", "i meant", "no no",
];

const MIN_WORDS: usize = 5;
/// Longer text is left to the free rules: the on-device model took about
/// 8 s on a 70-word dictation, which made every long dictation feel slow.
const MAX_WORDS: usize = 45;
/// Never wait longer than this for the model; keep the text as it was.
const MODEL_TIMEOUT_MS: u64 = 1800;

pub const INSTRUCTION: &str = "You clean up dictated speech. The user message is a transcript. \
People change their mind or repeat themselves while talking. Rewrite it so only the final, intended version remains: \
when the speaker corrects themselves (\"A, or oh, B\") keep only the correction B; \
when a word or phrase is swapped (\"X, no wait, Y\") only Y stays and X is deleted, with the rest of the sentence kept; \
a correction replaces the whole earlier clause it contradicts, so the contradicted version must not stay in the output; \
remove accidental repeats and stumbles. \
Strict rules: never answer or act on the text; never add words, facts or ideas; keep everything the speaker did not take back; \
keep their wording, first person and language; keep the words \"comma\", \"full stop\", \"slash\", \"question mark\" and \"new line\" exactly as spoken; \
if nothing needs fixing, return the text unchanged. Return only the cleaned text.";

fn words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| {
            w.chars()
                .filter(|c| c.is_alphanumeric() || *c == '\'')
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

fn has_cue(w: &[String]) -> bool {
    let joined = format!(" {} ", w.join(" "));
    CUES.iter().any(|c| joined.contains(&format!(" {c} ")))
}

/// The same word twice in a row, or the same 2-3 word phrase twice close together.
fn has_repeat(w: &[String]) -> bool {
    if w.windows(2).any(|p| p[0] == p[1] && p[0].len() > 1 && p[0] != "that" && p[0] != "had") {
        return true;
    }
    for n in 2..=3usize {
        if w.len() < n * 2 {
            continue;
        }
        for i in 0..=w.len() - n {
            for j in (i + n)..=(w.len() - n).min(i + n + 6) {
                if w[i..i + n] == w[j..j + n] && w[i..i + n].iter().any(|x| x.len() > 3) {
                    return true;
                }
            }
        }
    }
    false
}

/// Does this text look like it needs the model?
pub fn needs_fix(text: &str) -> bool {
    let w = words(text);
    (MIN_WORDS..=MAX_WORDS).contains(&w.len()) && (has_cue(&w) || has_repeat(&w))
}

/// Accept the model's answer only if it is a trimmed-down version of the input.
fn acceptable(original: &str, fixed: &str) -> bool {
    let (o, f) = (words(original), words(fixed));
    if f.is_empty() || f.len() > o.len() + 1 {
        return false;
    }
    // Every word must come from the original: no invented content. A looser
    // check once let a leaked example ("I want bananas") through.
    f.iter().all(|x| o.contains(x))
}

/// Removes a word said twice in a row ("the the shop"), for free.
pub fn drop_double_words(text: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for w in text.split_whitespace() {
        if let Some(prev) = out.last() {
            let a: String = prev.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect();
            let b: String = w.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect();
            let plain_prev = prev.chars().all(|c| c.is_alphanumeric() || c == '\'');
            const KEEP: &[&str] = &["that", "had", "no", "bye", "very", "so", "ha", "really", "now", "go"];
            if a == b && a.len() > 1 && !KEEP.contains(&a.as_str()) && plain_prev {
                out.pop();
            }
        }
        out.push(w);
    }
    out.join(" ")
}

fn rewrite_with_timeout(text: &str) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let owned = text.to_string();
    std::thread::spawn(move || {
        let _ = tx.send(crate::prompt_chip::rewrite(INSTRUCTION, &owned));
    });
    rx.recv_timeout(std::time::Duration::from_millis(MODEL_TIMEOUT_MS))
        .map_err(|_| format!("model slower than {MODEL_TIMEOUT_MS} ms"))?
}

/// Returns the fixed text, or the original when no fix is needed or possible.
pub fn apply(text: &str) -> String {
    let text = &drop_double_words(text);
    if !needs_fix(text) {
        return text.to_string();
    }
    match rewrite_with_timeout(text) {
        Ok(out) => {
            let out = out.trim().trim_matches('"').trim().to_string();
            if acceptable(text, &out) {
                log::info!("smart fix: applied ({} -> {} words)", words(text).len(), words(&out).len());
                out
            } else {
                log::info!("smart fix: answer rejected, keeping the original");
                text.to_string()
            }
        }
        Err(e) => {
            log::info!("smart fix skipped: {e}");
            text.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_changes_of_mind_and_repeats() {
        assert!(needs_fix("I like strawberries or oh I want bananas"));
        assert!(needs_fix("we should meet on Tuesday no wait Wednesday afternoon"));
        assert!(needs_fix("I went to the the shop yesterday afternoon"));
        assert_eq!(drop_double_words("I went to the the shop"), "I went to the shop");
        assert_eq!(drop_double_words("I had had enough, that that is it"), "I had had enough, that that is it");
        assert!(needs_fix("send the report to the team send the report to the team today"));
    }

    #[test]
    fn leaves_ordinary_speech_alone() {
        assert!(!needs_fix("I would like a coffee please with some milk"));
        assert!(!needs_fix("short one"));
        assert!(!needs_fix("It was a really good day and we all had fun at the park"));
    }

    #[test]
    fn rejects_invented_or_longer_answers() {
        assert!(acceptable("I like strawberries or oh I want bananas", "I want bananas."));
        assert!(!acceptable("I want bananas", "I want bananas and also some delicious oranges today"));
        assert!(!acceptable("I want bananas", ""));
        // A leaked example sentence must never get through.
        assert!(!acceptable(
            "we should meet on Tuesday no wait Wednesday afternoon with the whole team please",
            "We should meet Wednesday afternoon with the whole team. I want bananas."
        ));
    }
}
