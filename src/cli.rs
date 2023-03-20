use command_line;
use std::env;

#[cfg(target_os = "linux")]
fn start_service() {
    command_line::execute_command("systemctl --user start gh-notifier.timer", false);
    command_line::execute_command("systemctl --user enable gh-notifier.timer", false);
}

#[cfg(target_os = "linux")]
fn stop_service() {
    command_line::execute_command("systemctl --user stop gh-notifier.timer", false);
    command_line::execute_command("systemctl --user disable gh-notifier.timer", false);
}

#[cfg(target_os = "macos")]
fn start_service() -> bool {
    command_line::execute_command(
        "launchctl load $HOME/Library/LaunchAgents/com.gh-notifier.plist",
        false,
    )
}

#[cfg(target_os = "macos")]
fn stop_service() -> bool {
    command_line::execute_command(
        "launchctl unload $HOME/Library/LaunchAgents/com.gh-notifier.plist",
        false,
    )
}

pub fn get_args() -> bool {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return false;
    }
    match args[1].as_str() {
        "stop" => {
            stop_service();
            true
        }
        "start" => {
            start_service();
            true
        }
        _ => false,
    }
}
