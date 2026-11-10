//! Smart detection: is this dictation a prompt for an AI, or normal talk?
//!
//! Offline rules, no model call, so it costs nothing. Built to be careful:
//! wrongly rewriting an email or a chat message is worse than missing a
//! prompt, so anything unclear is pasted as spoken.
//!
//! The app the text is going into is the strongest signal (Claude, ChatGPT,
//! Cursor, a terminal). The wording only confirms it.

/// What the rules decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detected {
    /// Clearly a prompt for an AI: rewrite it.
    Prompt,
    /// Normal talk, or not sure: paste as spoken.
    Normal,
}

/// Fewer words than this are never rewritten on wording alone.
const MIN_WORDS: usize = 50;
/// In an AI app, text this long is rewritten when the wording is clearly a prompt.
const MIN_WORDS_CLEAR: usize = 30;
/// Score that counts as "clearly a prompt" (app bonus included).
const THRESHOLD_CLEAR: i32 = 7;

/// Spoken commands that always turn the dictation into a prompt, wherever
/// they are said ("... turn this into a prompt").
static TRIGGER: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
    regex::Regex::new(
        r"(?i)[,.;:!?]?\s*\b(?:(?:please|and|okay|ok|so)\s+)?(?:(?:turn|make|convert|change|rewrite)\s+(?:this|that|it|all\s+this|all\s+of\s+this|the\s+above)\s+(?:into\s+|in\s+to\s+|as\s+)?(?:a\s+)?(?:proper\s+|good\s+|clear\s+)?prompt|prompt[\s-]+engineer\s+(?:this|that|it))(?:\s+please)?\b[,.;:!?]?",
    )
    .expect("trigger regex")
});

/// If the speaker asked for a prompt out loud, returns the text without that
/// command. None when no command was spoken.
pub fn spoken_trigger(text: &str) -> Option<String> {
    if !TRIGGER.is_match(text) {
        return None;
    }
    let cleaned = TRIGGER.replace_all(text, " ");
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let cleaned = cleaned
        .trim_matches(|c: char| c == ',' || c == ';' || c == ':' || c.is_whitespace())
        .to_string();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}
/// Score needed in an AI app (the app already counts for +3).
const THRESHOLD_AI_APP: i32 = 5;
/// Score needed anywhere else (higher, so wording alone rarely wins).
const THRESHOLD_OTHER_APP: i32 = 7;

/// Apps where people talk to an AI: chat apps, AI code editors, terminals.
const AI_APPS: &[&str] = &[
    "com.anthropic.claudefordesktop",
    "com.openai.chat",
    "com.openai.codex",
    "com.todesktop.230313mzl4w4u92", // Cursor
    "com.exafunction.windsurf",
    "com.microsoft.vscode",
    "dev.zed.zed",
    "com.apple.terminal",
    "com.googlecode.iterm2",
    "dev.warp.warp-stable",
    "com.mitchellh.ghostty",
    "ai.perplexity.mac",
];

/// Apps for writing to people: never rewrite there.
const PEOPLE_APPS: &[&str] = &[
    "com.apple.mail",
    "com.apple.mobilesms",
    "com.apple.notes",
    "com.apple.iwork.pages",
    "com.microsoft.word",
    "com.microsoft.outlook",
    "com.tinyspeck.slackmacgap",
    "com.hnc.discord",
    "net.whatsapp.whatsapp",
    "desktop.whatsapp",
    "ru.keepcoder.telegram",
    "com.facebook.archon",
];

const TASK_VERBS: &[&str] = &[
    "build", "write", "make", "fix", "create", "generate", "implement", "refactor",
    "add", "explain", "summarize", "summarise", "design", "rewrite", "translate",
    "analyze", "analyse", "debug", "list", "draft", "find", "research", "update",
    "change", "remove", "convert", "review", "test", "check", "improve", "move",
    "rename", "install", "run",
];

const AI_ADDRESS: &[&str] = &[
    "i want you to", "i need you to", "can you", "could you", "please",
    "you should", "you are a", "act as", "your task", "make sure you",
    "make sure that", "don't forget to", "do not", "i'd like you to",
];

const SPEC_WORDS: &[&str] = &[
    "must", "should", "requirements", "format", "steps", "step", "using",
    "function", "file", "code", "app", "component", "feature", "output",
    "example", "markdown", "table", "bullet", "screen", "button", "api",
];

const CHAT_MARKERS: &[&str] = &[
    "hi ", "hey ", "hello ", "dear ", "thanks", "thank you", "lol", "haha",
    "love you", "miss you", "best regards", "kind regards", "cheers",
    "yesterday", "last night", "how are you", "see you", "talk soon",
];

fn words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'')
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect()
}

/// Is this app one where people talk to an AI?
pub fn is_ai_app(bundle_id: &str) -> bool {
    AI_APPS.contains(&bundle_id.to_lowercase().as_str())
}

/// Score the wording alone (no app bonus). Positive = prompt-like.
pub fn wording_score(text: &str) -> i32 {
    let lower = format!(" {} ", words(text).join(" "));
    let w = words(text);
    let mut score = 0;
    // Spoken prompts often open with "okay so I want you to go into...", so
    // the first verb can come late.
    if w.iter().take(15).any(|x| TASK_VERBS.contains(&x.as_str())) {
        score += 2;
    }
    // Several asks in one dictation ("fix..., add..., test...") reads as a prompt.
    if w.iter().filter(|x| TASK_VERBS.contains(&x.as_str())).count() >= 3 {
        score += 2;
    }
    if AI_ADDRESS.iter().any(|p| lower.contains(&format!(" {p} "))) {
        score += 2;
    }
    let spec = SPEC_WORDS
        .iter()
        .filter(|s| w.iter().any(|x| x == *s))
        .count() as i32;
    score += spec.min(2);
    score += match w.len() {
        n if n >= 120 => 2,
        n if n >= 60 => 1,
        _ => 0,
    };
    let chat = CHAT_MARKERS
        .iter()
        .filter(|m| lower.contains(&format!(" {} ", m.trim())))
        .count() as i32;
    score -= (chat * 2).min(4);
    score
}

/// Decide for a dictation going into the app with this bundle id.
pub fn detect(text: &str, bundle_id: Option<&str>) -> Detected {
    if spoken_trigger(text).is_some() {
        return Detected::Prompt;
    }
    let n = words(text).len();
    let app = bundle_id.unwrap_or("").to_lowercase();
    if PEOPLE_APPS.contains(&app.as_str()) {
        return Detected::Normal;
    }
    let in_ai_app = AI_APPS.contains(&app.as_str());
    let score = wording_score(text) + if in_ai_app { 3 } else { 0 };
    if in_ai_app && n >= MIN_WORDS_CLEAR && score >= THRESHOLD_CLEAR {
        return Detected::Prompt;
    }
    if n < MIN_WORDS {
        return Detected::Normal;
    }
    let needed = if in_ai_app { THRESHOLD_AI_APP } else { THRESHOLD_OTHER_APP };
    if score >= needed {
        Detected::Prompt
    } else {
        Detected::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLAUDE: Option<&str> = Some("com.anthropic.claudefordesktop");
    const MAIL: Option<&str> = Some("com.apple.mail");
    const SAFARI: Option<&str> = Some("com.apple.Safari");

    const PROMPT: &str = "Build a settings screen for the app with a dark mode toggle, \
        make sure the toggle slides smoothly and use the existing button component, \
        then add a section for notifications with three options, keep the spacing the same as the other pages, \
        write tests for each option and fix anything that breaks before you finish the change";

    #[test]
    fn prompt_in_ai_app_is_detected() {
        assert_eq!(detect(PROMPT, CLAUDE), Detected::Prompt);
        assert_eq!(
            detect("I want you to write a function that sorts the list of users by name and returns the first ten", Some("com.todesktop.230313mzl4w4u92")),
            Detected::Normal, "short requests are just cleaned, never turned into a prompt"
        );
    }

    #[test]
    fn never_rewrites_in_mail_or_chat_apps() {
        assert_eq!(detect(PROMPT, MAIL), Detected::Normal);
        assert_eq!(detect(PROMPT, Some("com.tinyspeck.slackmacgap")), Detected::Normal);
    }

    #[test]
    fn chatty_text_stays_normal_even_in_ai_app() {
        let chat = "Hey thanks so much for yesterday it was really lovely to see you and talk about the trip, see you soon";
        assert_eq!(detect(chat, CLAUDE), Detected::Normal);
    }

    #[test]
    fn short_text_is_never_rewritten() {
        assert_eq!(detect("Fix the button please", CLAUDE), Detected::Normal);
    }

    #[test]
    fn other_apps_need_strong_wording() {
        assert_eq!(detect(PROMPT, SAFARI), Detected::Normal);
        let strong = "I want you to build a feature for the app that must show a table of steps, \
            use the existing component and make sure the output is markdown with an example for each step \
            and a short explanation of the function, then test the code and fix anything that breaks \
            and keep it simple and readable for everyone on the team please";
        assert_eq!(detect(strong, SAFARI), Detected::Prompt);
    }

    #[test]
    fn spoken_command_always_makes_a_prompt() {
        let said = "look at the settings page and make the toggles slide, turn this into a prompt";
        assert_eq!(detect(said, MAIL), Detected::Prompt);
        assert_eq!(
            spoken_trigger(said).as_deref(),
            Some("look at the settings page and make the toggles slide")
        );
        assert_eq!(
            spoken_trigger("Make this a prompt. Fix the login bug and add tests").as_deref(),
            Some("Fix the login bug and add tests")
        );
        assert_eq!(spoken_trigger("prompt engineer this please, build a timer app").as_deref(), Some("build a timer app"));
        assert_eq!(spoken_trigger("I wrote a prompt for the team yesterday"), None);
        assert_eq!(spoken_trigger("turn this into a prompt"), None, "nothing left to rewrite");
    }

    #[test]
    fn clear_prompt_of_30_words_in_ai_app() {
        let clear = "I want you to build a settings screen for the app, make sure the toggle component slides \
            smoothly and the output must use the existing button code and test every step";
        assert_eq!(detect(clear, CLAUDE), Detected::Prompt);
        assert_eq!(detect(clear, SAFARI), Detected::Normal);
    }

    #[test]
    fn spoken_instructions_in_ai_apps_are_detected() {
        let murmur = "okay so I want you to go into the murmur app and make the transcription bar a bit wider \
            and also fix the shortcut because it clashes with claude and make sure you test it";
        assert_eq!(detect(murmur, CLAUDE), Detected::Prompt);
        let login = "um so can you look at the login page on the website, the password reset email never arrives, \
            I think it's the smtp settings, check the logs, fix it and then add a test so it doesn't break again, \
            and don't touch the sign up flow";
        assert_eq!(detect(login, CLAUDE), Detected::Prompt);
        assert_eq!(detect(login, Some("com.apple.Terminal")), Detected::Prompt);
        assert_eq!(detect(murmur, MAIL), Detected::Normal);
    }

    #[test]
    fn normal_email_text_stays_normal() {
        let email = "Can you send me the invoice by Friday, I need it for the accounts and the meeting with the bank next week";
        assert_eq!(detect(email, SAFARI), Detected::Normal);
    }
}
