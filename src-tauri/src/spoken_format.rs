//! Turns spoken formatting into symbols, offline: "full stop" -> ".",
//! "comma" -> ",", "slash" -> "/", "new line" -> line break, and number words
//! of ten or more ("twenty five") -> digits. Small numbers stay as words
//! ("two cats"), the way people write them.

/// Spoken phrase (lowercase words) -> symbol, and whether it joins the
/// previous word (no space before) and the next word (no space after).
const PUNCTUATION: [(&str, &str, bool, bool); 13] = [
    ("full stop", ".", true, false),
    ("comma", ",", true, false),
    ("question mark", "?", true, false),
    ("exclamation mark", "!", true, false),
    ("exclamation point", "!", true, false),
    ("colon", ":", true, false),
    ("semicolon", ";", true, false),
    ("forward slash", "/", false, true),
    ("slash", "/", false, true),
    ("new line", "\n", true, true),
    ("new paragraph", "\n\n", true, true),
    ("open bracket", "(", false, true),
    ("close bracket", ")", true, false),
];

/// Numbers below this stay as words.
const MIN_NUMBER_AS_DIGITS: u64 = 10;

fn norm(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn trailing_punct(word: &str) -> &str {
    let end = word.trim_end_matches(|c: char| ".,!?;:".contains(c));
    &word[end.len()..]
}

fn unit(word: &str) -> Option<u64> {
    let v = match word {
        "zero" => 0, "one" => 1, "two" => 2, "three" => 3, "four" => 4,
        "five" => 5, "six" => 6, "seven" => 7, "eight" => 8, "nine" => 9,
        "ten" => 10, "eleven" => 11, "twelve" => 12, "thirteen" => 13,
        "fourteen" => 14, "fifteen" => 15, "sixteen" => 16, "seventeen" => 17,
        "eighteen" => 18, "nineteen" => 19, "twenty" => 20, "thirty" => 30,
        "forty" => 40, "fifty" => 50, "sixty" => 60, "seventy" => 70,
        "eighty" => 80, "ninety" => 90,
        _ => return None,
    };
    Some(v)
}

fn scale(word: &str) -> Option<u64> {
    match word {
        "hundred" => Some(100),
        "thousand" => Some(1_000),
        "million" => Some(1_000_000),
        _ => None,
    }
}

/// Parses number words starting at `words[i]`; returns (value, words used).
fn parse_number(words: &[String], i: usize) -> Option<(u64, usize)> {
    let (mut total, mut current, mut used) = (0u64, 0u64, 0usize);
    let mut saw_number = false;
    while i + used < words.len() {
        let raw = norm(&words[i + used]);
        let parts: Vec<&str> = raw.split('-').collect();
        if parts.iter().all(|p| unit(p).is_some()) && !raw.is_empty() {
            current += parts.iter().filter_map(|p| unit(p)).sum::<u64>();
        } else if raw == "and" && saw_number && i + used + 1 < words.len()
            && unit(&norm(&words[i + used + 1])).is_some()
        {
            // "one hundred and five"
        } else if let Some(s) = scale(&raw) {
            if !saw_number {
                break;
            }
            if s == 100 {
                current = current.max(1) * 100;
            } else {
                total += current.max(1) * s;
                current = 0;
            }
        } else {
            break;
        }
        saw_number = true;
        used += 1;
        // Stop at a word that ends a clause ("twenty five, then").
        if !trailing_punct(&words[i + used - 1]).is_empty() {
            break;
        }
    }
    saw_number.then_some((total + current, used))
}

fn convert_numbers(words: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        match parse_number(&words, i) {
            Some((value, used)) if value >= MIN_NUMBER_AS_DIGITS => {
                let tail = trailing_punct(&words[i + used - 1]).to_string();
                out.push(format!("{value}{tail}"));
                i += used;
            }
            _ => {
                out.push(words[i].clone());
                i += 1;
            }
        }
    }
    out
}

fn match_punct(words: &[String], i: usize) -> Option<(&'static str, bool, bool, usize)> {
    PUNCTUATION.iter().find_map(|(phrase, sym, join_prev, join_next)| {
        let parts: Vec<&str> = phrase.split(' ').collect();
        let fits = i + parts.len() <= words.len()
            && parts.iter().enumerate().all(|(k, p)| norm(&words[i + k]) == *p);
        fits.then_some((*sym, *join_prev, *join_next, parts.len()))
    })
}

/// "slash" only becomes "/" when a word follows straight after it
/// ("the slash prompt skill" -> "the /prompt skill"); otherwise it stays a word.
fn slash_has_word_after(words: &[String], next: usize) -> bool {
    words
        .get(next)
        .and_then(|w| w.chars().next())
        .is_some_and(|c| c.is_alphanumeric())
        && match_punct(words, next).is_none()
}

fn capitalize(word: &str) -> String {
    let mut c = word.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

pub fn apply(text: &str) -> String {
    let words: Vec<String> = text.split_whitespace().map(str::to_string).collect();
    let words = convert_numbers(words);
    let mut out = String::with_capacity(text.len());
    let mut glue_next = false;
    let mut cap_next = false;
    let mut i = 0;
    while i < words.len() {
        if let Some((sym, join_prev, join_next, used)) = match_punct(&words, i)
            .filter(|(sym, _, _, used)| {
                *sym != "/"
                    || (trailing_punct(&words[i + used - 1]).is_empty()
                        && slash_has_word_after(&words, i + used))
            })
        {
            if join_prev {
                // Drop punctuation the model already put on the previous word.
                while out.ends_with([',', '.', ' ']) && !out.is_empty() && sym != "/" {
                    out.pop();
                }
            } else if !out.is_empty() && !glue_next && !out.ends_with('\n') {
                out.push(' ');
            }
            out.push_str(sym);
            glue_next = join_next;
            cap_next = matches!(sym, "." | "?" | "!" | "\n" | "\n\n");
            i += used;
            continue;
        }
        let word = words[i].trim_start_matches([',', '.']);
        let word = if cap_next { capitalize(word) } else { word.to_string() };
        if !out.is_empty() && !glue_next {
            out.push(' ');
        }
        out.push_str(&word);
        glue_next = false;
        cap_next = false;
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spoken_punctuation_becomes_symbols() {
        assert_eq!(apply("Hello comma how are you question mark"), "Hello, how are you?");
        assert_eq!(apply("It's done full stop thanks"), "It's done. Thanks");
        assert_eq!(apply("and slash or"), "and /or");
        assert_eq!(apply("use the slash prompt skill"), "use the /prompt skill");
        assert_eq!(apply("it ends with a slash"), "it ends with a slash");
        assert_eq!(apply("a slash, then more"), "a slash, then more");
        assert_eq!(apply("Hi. full stop"), "Hi.");
    }

    #[test]
    fn numbers_ten_and_up_become_digits() {
        assert_eq!(apply("I need twenty five apples"), "I need 25 apples");
        assert_eq!(apply("about one hundred and five people"), "about 105 people");
        assert_eq!(apply("two thousand twenty six"), "2026");
        assert_eq!(apply("I have two cats"), "I have two cats");
        assert_eq!(apply("twenty-one"), "21");
    }

    #[test]
    fn ordinary_text_is_unchanged() {
        let t = "A normal sentence, with punctuation. And more!";
        assert_eq!(apply(t), t);
        assert_eq!(apply("the period of time"), "the period of time");
    }
}
