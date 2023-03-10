use command_line::{command_exists, execute_command};
use std::process;

#[macro_use]
extern crate derive_builder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let notification = NotificationPartsBuilder::default()
            .title("TEST NOTIFICATION")
            .subtitle("Subtitle")
            .message("This is the message.")
            .sound("Pop")
            .open("https://google.com")
            .build()
            .expect("Could not build notification");

        notify(&notification)
        // you should see a notification
    }
}

const TERMINAL_NOTIFIER_UNSAFE_CHARS: [char; 2] = ['[', ']'];

#[derive(Default, Builder)]
pub struct NotificationParams<'notification> {
    #[builder(setter(into))]
    pub title: &'notification str,
    #[builder(setter(into))]
    pub subtitle: &'notification str,
    #[builder(setter(into))]
    pub message: &'notification str,
    #[builder(setter(strip_option), default)]
    pub sound: Option<&'notification str>,
    #[builder(setter(strip_option), default)]
    pub open: Option<&'notification str>,
}

pub fn notify(notification: &NotificationParams) {
    let title = notification.title;
    let subtitle = notification.subtitle;
    let message = notification.message;
    if cfg!(target_os = "macos") {
        let sound_str = match notification.sound {
            Some(s) => s,
            None => "default",
        };

        let open_str = match notification.open {
            Some(s) => s,
            None => "",
        };

        terminal_notifier_command(title, subtitle, message, sound_str, open_str);
    } else {
        notify_send_command(title, subtitle, message);
    }
}

fn terminal_notifier_command(title: &str, subtitle: &str, message: &str, sound: &str, open: &str) {
    // check terminal-notifier is installed
    if !command_exists("terminal-notifier -h") {
        println!("terminal-notifier is not available. Is it installed?");
        process::exit(1);
    }

    // escape chars not supported by terminal-notifier
    let mut safe_message = message.to_owned();
    for c in TERMINAL_NOTIFIER_UNSAFE_CHARS {
        safe_message = safe_message.replace(c, "")
    }

    // build MacOS terminal-notifier command line
    let mut notification_str = format!(
        "-title \"{title}\" \
         -subtitle \"{subtitle}\" \
         -message \"{safe_message}\" \
         -sound \"{sound}\""
    );
    if open.len() > 0 {
        notification_str = format!("{notification_str} -open \"{open}\"")
    }

    // execute the command
    execute_command(&format!("terminal-notifier {notification_str}"), true);
}

fn notify_send_command(title: &str, subtitle: &str, message: &str) {
    // check notify-send is installed
    if !command_exists("notify-send -h") {
        println!("notify-send is not available. Is it installed?");
        process::exit(1);
    }

    // build linux command line arguments
    // the notify-send api does not support on-click actions
    let notification_str = format!("\"{title} ({subtitle})\" \"{message}\"");

    // execute command
    execute_command(&format!("notify-send {notification_str}"), true);
}
