use std::env;
use std::fs::File;
use std::path::Path;

use command_line::execute_command;
use notify::NotificationBuilder;

const LAUNCH_AGENT_PLIST_PATH: &str = "$HOME/Library/LaunchAgents/com.gh-notifier.plist";

fn get_args() -> Vec<String> {
    env::args().collect()
}

fn stop_service() -> bool {
    execute_command(
        &format!("launchctl unload {LAUNCH_AGENT_PLIST_PATH}"),
        false,
    )
}

fn start_service() -> bool {
    execute_command(&format!("launchctl load {LAUNCH_AGENT_PLIST_PATH}"), false)
}

pub fn parse_args() -> bool {
    let args = get_args();
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

pub fn build_pull_or_issue_url(url: Option<String>) -> String {
    // url returned is for the api, we need to build the html url
    match url {
        Some(url) => {
            let url_parts = url.split("/").collect::<Vec<&str>>();
            let len_url_parts = url_parts.len();
            let pull_or_issue = if url_parts.contains(&"issues") {
                "issues"
            } else {
                "pull"
            };
            format!(
                "https://github.com/{}/{}/{}/{}",
                url_parts[len_url_parts - 4], // user/org
                url_parts[len_url_parts - 3], // repo
                pull_or_issue,                // pull/issues
                url_parts[len_url_parts - 1]  // #number
            )
        }
        None => "".to_string(),
    }
}

pub fn get_persistence_file_path() -> String {
    let mut ids_file_path = env::var("HOME").expect("$HOME environment variable is not set");
    let ids_filename = "/.gh-read-notification-ids";
    ids_file_path.push_str(ids_filename);
    if !Path::new(&ids_file_path).exists() {
        File::create(&ids_file_path).expect("creating persistent ids file failed");
    }
    ids_file_path
}

pub fn notify_error(error: &str) {
    match NotificationBuilder::default()
        .title("Github Notifier")
        .subtitle("Error")
        .message(error)
        .sound("Pop")
        .build()
    {
        Ok(n) => n.notify(),
        Err(err) => {
            dbg!(err);
        }
    }
}

pub fn display_new_github_notification(message: &str, onclick_url: &str, subtitle: &str) {
    match NotificationBuilder::default()
        .title("New Github Notification")
        .subtitle(subtitle)
        .message(message)
        .open(onclick_url)
        .build()
    {
        Ok(n) => n.notify(),
        Err(err) => {
            dbg!(err);
        }
    }
}
