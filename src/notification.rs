use std::time::Duration;

use notify_rust::{Notification, Urgency};

const UP_ICON: &str = "/usr/share/icons/HighContrast/32x32/actions/go-up.png";
const DOWN_ICON: &str = "/usr/share/icons/HighContrast/32x32/actions/go-down.png";

const UP_TITLE: &str = "Stand up!";
const UP_MESSAGE: &str = "   ↑       ↑       ↑
  ↑↑↑     ↑↑↑     ↑↑↑
 ↑↑↑↑↑   ↑↑↑↑↑   ↑↑↑↑↑
↑↑↑↑↑↑↑ ↑↑↑↑↑↑↑ ↑↑↑↑↑↑↑";

const DOWN_TITLE: &str = "Sit down!";
const DOWN_MESSAGE: &str = "↓↓↓↓↓↓↓ ↓↓↓↓↓↓↓ ↓↓↓↓↓↓↓
  ↓↓↓↓↓   ↓↓↓↓↓   ↓↓↓↓↓
   ↓↓↓     ↓↓↓     ↓↓↓
    ↓       ↓       ↓";

pub fn send_stand_notification(duration: Duration) {
    let stand_up_end_time = format_time_after_duration(duration);
    let message = format!("Stand up until {} \n{}", stand_up_end_time, UP_MESSAGE);

    Notification::new()
        .body(message.as_str())
        .icon(UP_ICON)
        .summary(UP_TITLE)
        .urgency(Urgency::Critical)
        .show()
        .unwrap();
}

pub fn send_sit_notification(duration: Duration) {
    let sit_down_end_time = format_time_after_duration(duration);
    let message = format!("Sit down until {} \n {}", sit_down_end_time, DOWN_MESSAGE);

    Notification::new()
        .body(message.as_str())
        .icon(DOWN_ICON)
        .summary(DOWN_TITLE)
        .urgency(Urgency::Critical)
        .show()
        .unwrap();
}

const LONG_TIME_FORMAT: &str = "%H:%M:%S";

fn format_time_after_duration(duration: Duration) -> String {
    let sleep_time = duration.as_secs();

    let wait_time_delta: chrono::TimeDelta = chrono::TimeDelta::try_seconds(sleep_time.try_into().unwrap_or_default())
        .unwrap_or_default();
    let sleep_end_time = chrono::Local::now()
        .checked_add_signed(wait_time_delta)
        .unwrap_or_default();

    sleep_end_time.format(LONG_TIME_FORMAT).to_string()
}
