//! Usage statistics computed from local dictation history. Nothing leaves
//! the Mac: words are counted from saved transcripts, speed from the length
//! of the saved recordings.

use chrono::{DateTime, Duration, FixedOffset, NaiveDate};
use serde::Serialize;
use specta::Type;
use std::collections::BTreeSet;

/// Days counted as "this week" (a rolling window ending now).
const WEEK_DAYS: i64 = 7;
/// Recordings shorter than this are left out of the speed figure.
const MIN_AUDIO_SECS: f64 = 1.0;
const SECS_PER_MINUTE: f64 = 60.0;

/// One past dictation.
#[derive(Debug, Clone, PartialEq)]
pub struct StatsSample {
    /// Unix seconds.
    pub timestamp: i64,
    pub words: u32,
    /// Length of the recording, when it is still on disk.
    pub audio_secs: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct UsageStats {
    pub total_words: u32,
    pub words_this_week: u32,
    pub dictations: u32,
    /// Average speaking speed; `None` without usable recordings.
    pub words_per_minute: Option<u32>,
    /// Days in a row with at least one dictation, ending today (or yesterday
    /// when nothing was dictated yet today).
    pub day_streak: u32,
}

pub fn word_count(text: &str) -> u32 {
    u32::try_from(text.split_whitespace().count()).unwrap_or(u32::MAX)
}

pub fn compute(samples: &[StatsSample], now: DateTime<FixedOffset>) -> UsageStats {
    let week_start = (now - Duration::days(WEEK_DAYS)).timestamp();
    let total_words = samples.iter().fold(0u32, |sum, s| sum.saturating_add(s.words));
    let words_this_week = samples
        .iter()
        .filter(|s| s.timestamp > week_start && s.timestamp <= now.timestamp())
        .fold(0u32, |sum, s| sum.saturating_add(s.words));
    UsageStats {
        total_words,
        words_this_week,
        dictations: u32::try_from(samples.len()).unwrap_or(u32::MAX),
        words_per_minute: words_per_minute(samples),
        day_streak: day_streak(samples, now),
    }
}

fn words_per_minute(samples: &[StatsSample]) -> Option<u32> {
    let (words, secs) = samples
        .iter()
        .filter_map(|s| s.audio_secs.filter(|a| *a >= MIN_AUDIO_SECS).map(|a| (s.words, a)))
        .fold((0f64, 0f64), |(w, t), (sw, st)| (w + f64::from(sw), t + st));
    if secs <= 0.0 {
        return None;
    }
    Some((words / (secs / SECS_PER_MINUTE)).round() as u32)
}

fn local_day(timestamp: i64, now: &DateTime<FixedOffset>) -> Option<NaiveDate> {
    DateTime::from_timestamp(timestamp, 0).map(|t| t.with_timezone(&now.timezone()).date_naive())
}

fn day_streak(samples: &[StatsSample], now: DateTime<FixedOffset>) -> u32 {
    let days: BTreeSet<NaiveDate> = samples
        .iter()
        .filter_map(|s| local_day(s.timestamp, &now))
        .collect();
    let today = now.date_naive();
    let mut day = if days.contains(&today) {
        today
    } else {
        match today.pred_opt() {
            Some(yesterday) => yesterday,
            None => return 0,
        }
    };
    let mut streak = 0;
    while days.contains(&day) {
        streak += 1;
        match day.pred_opt() {
            Some(prev) => day = prev,
            None => break,
        }
    }
    streak
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const DAY: i64 = 86_400;

    fn now() -> DateTime<FixedOffset> {
        FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2026, 11, 8, 12, 0, 0)
            .unwrap()
    }

    fn sample(days_ago: i64, words: u32, audio_secs: Option<f64>) -> StatsSample {
        StatsSample {
            timestamp: now().timestamp() - days_ago * DAY,
            words,
            audio_secs,
        }
    }

    #[test]
    fn empty_history_gives_zeroes() {
        let stats = compute(&[], now());
        assert_eq!(stats.total_words, 0);
        assert_eq!(stats.words_per_minute, None);
        assert_eq!(stats.day_streak, 0);
        assert_eq!(stats.dictations, 0);
    }

    #[test]
    fn counts_total_and_this_week() {
        let samples = [sample(0, 10, None), sample(6, 5, None), sample(8, 100, None)];
        let stats = compute(&samples, now());
        assert_eq!(stats.total_words, 115);
        assert_eq!(stats.words_this_week, 15);
        assert_eq!(stats.dictations, 3);
    }

    #[test]
    fn speed_uses_only_real_recordings() {
        // 150 words over 60 s = 150 wpm; the 0.5 s clip and the missing file are ignored.
        let samples = [
            sample(0, 100, Some(40.0)),
            sample(0, 50, Some(20.0)),
            sample(0, 9, Some(0.5)),
            sample(0, 999, None),
        ];
        assert_eq!(compute(&samples, now()).words_per_minute, Some(150));
    }

    #[test]
    fn streak_counts_days_in_a_row() {
        let samples = [sample(0, 1, None), sample(1, 1, None), sample(2, 1, None), sample(4, 1, None)];
        assert_eq!(compute(&samples, now()).day_streak, 3);
    }

    #[test]
    fn streak_survives_until_today_is_over() {
        let samples = [sample(1, 1, None), sample(2, 1, None)];
        assert_eq!(compute(&samples, now()).day_streak, 2);
    }

    #[test]
    fn streak_breaks_after_a_missed_day() {
        let samples = [sample(2, 1, None), sample(3, 1, None)];
        assert_eq!(compute(&samples, now()).day_streak, 0);
    }

    #[test]
    fn counts_words_by_whitespace() {
        assert_eq!(word_count("  hello   big\nworld "), 3);
        assert_eq!(word_count(""), 0);
    }
}
