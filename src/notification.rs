use notify_rust::{Notification, Urgency};

const UP_ICON : &str = "/usr/share/icons/HighContrast/32x32/actions/go-up.png"; 
const DOWN_ICON : &str = "/usr/share/icons/HighContrast/32x32/actions/go-down.png";

pub fn send_stand_notification() {
    Notification::new()
        .body("We're the bloody Ubersreik five... or four, doesn't matter")
        .icon(UP_ICON)
        .urgency(Urgency::Critical)
        .appname("STAND UP")
        .show()
        .unwrap();
}

pub fn send_sit_notification() {
    Notification::new()
        .body("We're the bloody Ubersreik five... or four, doesn't matter")
        .icon(DOWN_ICON)
        .urgency(Urgency::Critical)
        .appname("SIT DOWN")
        .show()
        .unwrap();
}