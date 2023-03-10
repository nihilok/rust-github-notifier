use command_line::execute_command;
use notify::{notify, NotificationParamsBuilder};
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Client, Error};
use serde::Deserialize;
use std::fs::File;
use std::path::Path;
use std::{env, fs, process};

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
            notify_error(&error_text);
            println!("{}", error_text);
            process::exit(1);
        }
    };

    // get or create local persistence file to save notification ids already shown
    let ids_file_path = get_persistence_file_path();

    // make request to GH notifications API
    let client = Client::new();
    let response = match client
        .get(REQUEST_URL)
        .header(USER_AGENT, "Rust Reqwest")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(ACCEPT, "application/vnd.github+json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            notify_connection_error(&format!("{e}"));
            println!("{}", e);
            process::exit(1);
        }
    };

    // handle unsuccessful responses
    let status = response.status();
    if status != 200 {
        let text = response.text().await?;
        let detail = text.split(' ').collect::<String>();
        notify_connection_error(&format!("{status} {detail}"));
        println!("Error response: {} {}", status, text);
        process::exit(1);
    };

    // read already notified ids from file
    let read_ids_str = fs::read_to_string(&ids_file_path).expect("could not read ids from file");
    let read_id_strs = read_ids_str.split(",").collect::<Vec<&str>>();
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
        let message = notification.subject.title.as_str();
        let optional_url = notification.subject.url.clone();
        let onclick_url = match optional_url {
            Some(url) => {
                let pull_issue_url = build_pull_or_issue_url(&url);
                pull_issue_url
            },
            None => "".to_string(),
        };
        let subtitle = (&notification
            .reason
            .split("_")
            .collect::<Vec<&str>>()
            .join(" ")).to_owned();

        let params = NotificationParamsBuilder::default()
            .title("New Github Notification")
            .subtitle(subtitle.as_str())
            .message(message)
            .open(onclick_url.as_str())
            .build()
            .expect("Failed to build Github notification parts");

        // display notification
        notify(&params);
    }

    // save notified IDs to persistence file
    let ids_len = new_ids.len();
    if ids_len == 1 {
        fs::write(&ids_file_path, &new_ids[0]).expect("Unable to write ids to file");
    } else if ids_len > 1 {
        let ids_to_write: String = new_ids.iter().map(|id| id.to_string() + ",").collect();
        fs::write(&ids_file_path, ids_to_write).expect("Unable to write ids to file");
    }
    Ok(())
}

fn build_pull_or_issue_url(url: &String) -> String {
    // url returned is for the api, we need to build the html url
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

fn get_persistence_file_path() -> String {
    let mut ids_file_path = env::var("HOME").expect("$HOME environment variable is not set");
    let ids_filename = "/.gh-read-notification-ids";
    ids_file_path.push_str(ids_filename);
    if !Path::new(&ids_file_path).exists() {
        File::create(&ids_file_path).expect("creating persistent ids file failed");
    }
    ids_file_path
}

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
        _ => false,
    }
}

fn notify_error(error: &str) {
    let params = NotificationParamsBuilder::default()
        .title("Github Notifier")
        .subtitle("Error")
        .message(error)
        .sound("Pop")
        .build()
        .expect("Could not build error notification");
    notify(&params)
}

fn notify_connection_error(detail: &str) {
    let error_text: String = format!("Response: {}", detail);
    notify_error(&error_text)
}
