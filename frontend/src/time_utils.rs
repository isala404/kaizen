use chrono::{DateTime, Datelike, Local, Utc};

pub fn parse_iso_to_epoch_secs(iso: &str) -> f64 {
    DateTime::parse_from_rfc3339(iso)
        .map(|dt| dt.timestamp() as f64)
        .unwrap_or(0.0)
}

pub fn now_secs() -> f64 {
    Utc::now().timestamp() as f64
}

pub fn focused_elapsed(time_spent_secs: i64, updated_at: &str) -> i64 {
    let updated_epoch = parse_iso_to_epoch_secs(updated_at);
    let elapsed = (now_secs() - updated_epoch).max(0.0) as i64;
    time_spent_secs + elapsed
}

pub fn format_timer(secs: i64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if d > 0 {
        format!("{d}d {h}:{m:02}:{s:02}")
    } else if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

pub fn format_duration(secs: i64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    if d > 0 {
        format!("{d}d {h}h")
    } else if h > 0 {
        format!("{h}h {m:02}m")
    } else if m > 0 {
        format!("{m}m")
    } else if secs > 0 {
        format!("{secs}s")
    } else {
        String::new()
    }
}

pub fn relative_time(iso: &str) -> String {
    let epoch = parse_iso_to_epoch_secs(iso);
    if epoch == 0.0 {
        return "just now".into();
    }
    let delta = (now_secs() - epoch).max(0.0) as i64;
    match delta {
        0..=59 => "just now".into(),
        60..=3599 => format!("{}m ago", delta / 60),
        3600..=86399 => format!("{}h ago", delta / 3600),
        86400..=172799 => "yesterday".into(),
        _ => format!("{} days ago", delta / 86400),
    }
}

pub fn today_start_epoch() -> f64 {
    let now = Local::now();
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|naive| naive.and_local_timezone(Local).single())
        .map(|dt| dt.timestamp() as f64)
        .unwrap_or(0.0)
}

pub fn week_start_epoch() -> f64 {
    let now = Local::now();
    let days_since_monday = now.weekday().num_days_from_monday();
    let monday = now.date_naive() - chrono::Duration::days(days_since_monday as i64);
    monday
        .and_hms_opt(0, 0, 0)
        .and_then(|naive| naive.and_local_timezone(Local).single())
        .map(|dt| dt.timestamp() as f64)
        .unwrap_or(0.0)
}
