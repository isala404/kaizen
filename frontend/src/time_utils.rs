/// Parse ISO 8601 timestamp to epoch seconds using js_sys::Date.
pub fn parse_iso_to_epoch_secs(iso: &str) -> f64 {
    let ms = js_sys::Date::parse(iso);
    if ms.is_nan() {
        return 0.0;
    }
    ms / 1000.0
}

/// Current epoch seconds.
pub fn now_secs() -> f64 {
    js_sys::Date::now() / 1000.0
}

/// Compute live elapsed seconds for a focused task.
pub fn focused_elapsed(time_spent_secs: i64, updated_at: &str) -> i64 {
    let base = time_spent_secs;
    let updated_epoch = parse_iso_to_epoch_secs(updated_at);
    let elapsed = (now_secs() - updated_epoch).max(0.0) as i64;
    base + elapsed
}

/// Format seconds as d:hh:mm:ss / h:mm:ss / m:ss for live timer display.
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

/// Format seconds as compact duration (e.g. "2h 14m", "1d 3h").
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

/// Relative time string like "just now", "5m ago", "2h ago", "yesterday", "3d ago".
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

/// Start of today (local time) as epoch seconds.
pub fn today_start_epoch() -> f64 {
    let now = js_sys::Date::new_0();
    let today = js_sys::Date::new_with_year_month_day(
        now.get_full_year(),
        now.get_month() as i32,
        now.get_date() as i32,
    );
    today.get_time() / 1000.0
}

/// Start of current week (Monday) as epoch seconds.
pub fn week_start_epoch() -> f64 {
    let now = js_sys::Date::new_0();
    let day = now.get_day(); // 0=Sunday
    let days_since_monday = if day == 0 { 6 } else { day - 1 };
    let today = js_sys::Date::new_with_year_month_day(
        now.get_full_year(),
        now.get_month() as i32,
        now.get_date() as i32,
    );
    (today.get_time() - (days_since_monday as f64 * 86_400_000.0)) / 1000.0
}
