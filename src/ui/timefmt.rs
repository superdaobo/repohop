use chrono::{DateTime, Utc};

/// Human-readable relative time for the project table.
pub fn format_relative_time(now: DateTime<Utc>, when: Option<DateTime<Utc>>) -> String {
    let Some(when) = when else {
        return "never".into();
    };
    let secs = now.signed_duration_since(when).num_seconds();
    if secs < 0 {
        // Clock skew / future timestamps — treat as recent.
        return "just now".into();
    }
    if secs < 60 {
        return "just now".into();
    }
    let mins = secs / 60;
    if mins < 60 {
        return format!("{mins}m ago");
    }
    let hours = mins / 60;
    if hours < 48 {
        return format!("{hours}h ago");
    }
    let days = hours / 24;
    if days < 60 {
        return format!("{days}d ago");
    }
    let months = days / 30;
    if months < 24 {
        return format!("{months}mo ago");
    }
    let years = days / 365;
    format!("{years}y ago")
}

/// Truncate a display path for a fixed column width (chars, not bytes).
pub fn truncate_path(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    if max_chars <= 1 {
        return "…".into();
    }
    // Prefer keeping the end of the path (more distinctive).
    let skip = count.saturating_sub(max_chars - 1);
    let tail: String = s.chars().skip(skip).collect();
    format!("…{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(secs, 0).unwrap()
    }

    #[test]
    fn never_when_none() {
        assert_eq!(format_relative_time(t(1_000), None), "never");
    }

    #[test]
    fn just_now_under_a_minute() {
        assert_eq!(format_relative_time(t(1_030), Some(t(1_000))), "just now");
    }

    #[test]
    fn minutes_and_hours() {
        assert_eq!(
            format_relative_time(t(1_000 + 5 * 60), Some(t(1_000))),
            "5m ago"
        );
        assert_eq!(
            format_relative_time(t(1_000 + 3 * 3600), Some(t(1_000))),
            "3h ago"
        );
    }

    #[test]
    fn days() {
        assert_eq!(
            format_relative_time(t(1_000 + 2 * 86_400), Some(t(1_000))),
            "2d ago"
        );
    }

    #[test]
    fn truncate_keeps_tail() {
        let s = r"D:\Documents\C_learn\repohop";
        let t = truncate_path(s, 12);
        assert!(t.starts_with('…'));
        assert!(t.ends_with("repohop") || t.ends_with("pohop") || t.contains("repohop"));
        assert!(t.chars().count() <= 12);
    }

    #[test]
    fn truncate_short_unchanged() {
        assert_eq!(truncate_path("abc", 10), "abc");
    }
}
