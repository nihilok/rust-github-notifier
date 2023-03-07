use command_line::execute_command;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        notify(
            "TEST NOTIFICATION",
            "Subtitle",
            "This is the message.",
            "Pop",
            "https://google.com",
        )
    }
}

const TERMINAL_NOTIFIER_UNSAFE_CHARS: [char; 2] = ['[', ']'];


pub fn notify(title: &str, subtitle: &str, message: &str, sound: &str, open: &str) {
    if cfg!(target_os = "linux") {
        notify_send_command(title, subtitle, message);
    } else {
        terminal_notifier_command(title, subtitle, message, sound, open);
    }
}

fn terminal_notifier_command(title: &str, subtitle: &str, message: &str, sound: &str, open: &str) {
    // escape chars not supported by terminal-notifier
    let mut safe_message = message.to_owned();
    for c in TERMINAL_NOTIFIER_UNSAFE_CHARS
    { safe_message = safe_message.replace(c, "") };

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
    // build linux command line arguments
    // the notify-send api does not permit on click actions, `open` and `sound` are unused
    let notification_str = format!("\"{title} ({subtitle})\" \"{message}\"");

    // execute command
    execute_command(&format!("notify-send {notification_str}"), true);
}
