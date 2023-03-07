use std::{env, fs, process};
use std::fs::File;
use std::path::Path;
use std::process::{Command, Output};
use serde::Deserialize;
use reqwest::{Client, Error};
use reqwest::header::{AUTHORIZATION, ACCEPT, USER_AGENT};

#[derive(Deserialize)]
struct NotificationSubject {
    title: String,
    url: Option<String>,
}

#[derive(Deserialize)]
struct Notification {
    id: String,
    subject: NotificationSubject,
    reason: String,
    updated_at: String,
}

const REQUEST_URL: &str = "https://api.github.com/notifications";
const ENV_VAR_NAME: &str = "GH_NOTIFIER_TOKEN";
const TERMINAL_NOTIFIER_UNSAFE_CHARS: [char; 2] = ['[', ']'];
const LAUNCH_AGENT_PLIST_PATH: &str = "$HOME/Library/LaunchAgents/com.gh-notifier.plist";

#[tokio::main]
async fn main() -> Result<(), Error> {
    // handle command line arguments
    if parse_args() {
        return Ok(());
    }
    // else if no arguments used, proceed with default actions:

    // get token from environment variable
    let token = match env::var(ENV_VAR_NAME) {
        Ok(t) => t,
        Err(e) => {
            let error_text = format!("{} {}", ENV_VAR_NAME, e);
            notify_error(&error_text).await;
            println!("{}", error_text);
            process::exit(1);
        }
    };

    // get or create local persistence file to save notification ids already shown
    let ids_file_path = get_persistence_file_path();

    // make request to GH notifications API
    let client = Client::new();
    let response = match client.get(REQUEST_URL)
        .header(USER_AGENT, "Rust Reqwest")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(ACCEPT, "application/vnd.github+json")
        .send().await {
        Ok(r) => r,
        Err(e) => {
            connection_error(&format!("{e}")).await;
            println!("{}", e);
            process::exit(1);
        }
    };

    // handle unsuccessful responses
    let status = response.status();
    if status != 200 {
        let text = response.text().await?;
        let detail = text.split(' ').collect::<String>();
        connection_error(&format!("{status} {detail}")).await;
        println!("Error response: {} {}", status, text);
        process::exit(1);
    };

    // read already notified ids from file
    let read_ids_str = fs::read_to_string(&ids_file_path)
        .expect("could not read ids from file");
    let read_id_strs = read_ids_str
        .split(",")
        .collect::<Vec<&str>>();
    let mut new_ids: Vec<String> = Vec::new();

    // handle successful API response
    let response_json: Vec<Notification> = response.json().await?;

    // loop through notifications in response, checking against saved notification ids
    // and display desktop notification if identifier not already saved to file
    for notification in &response_json {
        let mut identifier: String = notification.id.to_owned();
        identifier.push_str(&notification.updated_at);
        let check = identifier.clone();
        new_ids.push(identifier);
        if read_id_strs.contains(&check.as_str()) {
            // have already notified about this notification
            continue;
        }

        // build notification parts
        let title = &notification.subject.title;
        let optional_url = &notification.subject.url;
        let onclick_url: String = match optional_url {
            Some(url) => {
                build_pull_or_issue_url(url)
            }
            None => "".to_string()
        };
        let reason = &notification.reason;
        let reason = &reason
            .split("_")
            .collect::<Vec<&str>>()
            .join(" ");

        // display notification
        notify(
            "New Github Notification",
            reason,
            title,
            "Glass",
            &onclick_url,
        ).await;
    }

    // save notified IDs to persistence file
    let ids_len = new_ids.len();
    if ids_len == 1 {
        fs::write(&ids_file_path, &new_ids[0])
            .expect("Unable to write ids to file");
    } else if ids_len > 1 {
        let ids_to_write: String = new_ids.iter()
            .map(|id| id.to_string() + ",")
            .collect();
        fs::write(&ids_file_path, ids_to_write)
            .expect("Unable to write ids to file");
    }
    Ok(())
}

fn build_pull_or_issue_url(url: &String) -> String {
    let url_parts = url.split("/").collect::<Vec<&str>>();
    let len_url_parts = url_parts.len();
    let pull_or_issue = if url_parts.contains(&"issue") {
        "issue"
    } else {
        "pull"
    };
    format!(
        "https://github.com/{}/{}/{}/{}",
        url_parts[len_url_parts - 4],
        url_parts[len_url_parts - 3],
        pull_or_issue,
        url_parts[len_url_parts - 1]
    )
}

fn parse_args() -> bool {
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
        _ => {
            false
        }
    }
}

fn get_error_string(err: Vec<u8>, line: Option<&str>) -> String {
    let mut err_str=  String::from_utf8(err)
        .expect("Could not decode stderr");
    if let Some(l) = line {
        err_str.push_str(&format!("\nline {l}"))
    }
    err_str
}

fn display_error(command: Output, line: Option<&str>) {
    let err = command.stderr;
    if err.len() == 0 {
        return;
    }
    let err_string = get_error_string(err.clone(), line);
    if err_string.contains("Input/output error") {
        // launchd service was already stopped/started
        return;
    }
    if err_string.len() > 0 {
        println!("{}", err_string)
    }
}

fn get_persistence_file_path() -> String {
    let mut ids_file_path = env::var("HOME")
        .expect("$HOME environment variable is not set");
    let ids_filename = "/.gh-read-notification-ids";
    ids_file_path.push_str(ids_filename);
    if !Path::new(&ids_file_path).exists() {
        File::create(&ids_file_path)
            .expect("creating persistent ids file failed");
    }
    ids_file_path
}

async fn notify(title: &str, subtitle: &str, message: &str, sound: &str, open: &str) {
    let command: Output;
    if cfg!(target_os = "linux") {
        // build linux command line arguments
        // the notify-send api does not permit on click actions, `open` and `sound` are unused
        let notification_str = format!("\"{title} ({subtitle})\" \"{message}\"");

        // execute command
        command = Command::new("sh")
            .arg("-c")
            .arg(format!("notify-send {notification_str}"))
            .output()
            .expect("failed to execute notify-send process");
    } else {
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
        command = Command::new("sh")
            .arg("-c")
            .arg(format!("terminal-notifier {notification_str}"))
            .output()
            .expect("failed to execute terminal-notifier process");
    }

    // handle stderr
    let err = command.stderr;
    if err.len() > 0 {
        let err_disp = get_error_string(err, Some("230"));
        println!("{}", err_disp);
        process::exit(1);
    }
}

async fn notify_error(error: &str) {
    notify(
        "GitHub Notifier",
        "Error",
        error,
        "Pop",
        "",
    ).await
}

async fn connection_error(detail: &str) {
    let error_text: String = format!("Response: {}", detail);
    notify_error(&error_text).await
}

fn get_args() -> Vec<String> {
    env::args().collect()
}

fn stop_service() {
    let command = Command::new("sh")
        .arg("-c")
        .arg(format!("launchctl unload {LAUNCH_AGENT_PLIST_PATH}"))
        .output()
        .expect("failed to unload launch agent");
    display_error(command, Some("263"));
}

fn start_service() {
    let command = Command::new("sh")
        .arg("-c")
        .arg(format!("launchctl load {LAUNCH_AGENT_PLIST_PATH}"))
        .output()
        .expect("failed to load launch agent");
    display_error(command, Some("272"));
}
