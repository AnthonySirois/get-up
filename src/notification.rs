use notify_rust::{Notification, Urgency};

const UP_ICON : &str = "/usr/share/icons/HighContrast/32x32/actions/go-up.png"; 
const DOWN_ICON : &str = "/usr/share/icons/HighContrast/32x32/actions/go-down.png";

const UP_MESSAGE : &str = 
"   ↑       ↑       ↑
  ↑↑↑     ↑↑↑     ↑↑↑
 ↑↑↑↑↑   ↑↑↑↑↑   ↑↑↑↑↑
↑↑↑↑↑↑↑ ↑↑↑↑↑↑↑ ↑↑↑↑↑↑↑";

const DOWN_MESSAGE : &str = 
"↓↓↓↓↓↓↓ ↓↓↓↓↓↓↓ ↓↓↓↓↓↓↓
 ↓↓↓↓↓   ↓↓↓↓↓   ↓↓↓↓↓
  ↓↓↓     ↓↓↓     ↓↓↓
   ↓       ↓       ↓";

pub fn send_stand_notification() {
    Notification::new()
        .body(UP_MESSAGE)
        .icon(UP_ICON)
        .urgency(Urgency::Critical)
        .summary("Stand up!")
        .show()
        .unwrap();
}

pub fn send_sit_notification() {
    Notification::new()
        .body(DOWN_MESSAGE)
        .icon(DOWN_ICON)
        .urgency(Urgency::Critical)
        .summary("Sit down!")
        .show()
        .unwrap();
}