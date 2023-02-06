use std::{env, fs, process};
use std::fs::File;
use std::path::Path;
use std::process::{Command, Output};
use serde::Deserialize;
use reqwest::{Client, Error};
use reqwest::header::{AUTHORIZATION, ACCEPT, USER_AGENT};
// use serde_json::Value;

#[derive(Deserialize, Debug)]
struct NotificationSubject {
    title: String,
    url: String,
}

#[derive(Deserialize, Debug)]
struct Notification {
    id: String,
    subject: NotificationSubject,
    reason: String,
}

const REQUEST_URL: &str = "https://api.github.com/notifications";
const ENV_VAR_NAME: &str = "GH_NOTIFIER_TOKEN";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = match env::var(ENV_VAR_NAME) {
        Ok(t) => t,
        Err(e) => {
            let error_text = format!("{} {}", ENV_VAR_NAME, e);
            error(&error_text).await;
            println!("{}", error_text);
            process::exit(1);
        }
    };
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
    let status = response.status();
    if status != 200 {
        connection_error(&format!("{status}")).await;
        println!("Error connecting to GitHub API: {}", status);
        process::exit(1);
    };
    let response_json: Vec<Notification> = response.json().await?;
    let mut ids_file_path = env::var("HOME").expect("$HOME environment variable is not set");
    let ids_filename = "/.gh-read-notification-ids";
    ids_file_path.push_str(ids_filename);
    if !Path::new(&ids_file_path).exists() {
        File::create(&ids_file_path).expect("creating persistent ids file failed");
    }
    let read_ids_str = fs::read_to_string(&ids_file_path).expect("could not read ids from file");
    let read_id_strs = read_ids_str.split(",").collect::<Vec<&str>>();
    let mut new_ids: Vec<&str> = Vec::new();
    for notification in &response_json {
        new_ids.push(&notification.id);
        if !read_id_strs.contains(&&**&notification.id) {
            let title = &notification.subject.title;
            let reason = &notification.reason;
            let url = &notification.subject.url;
            let split_url = url.split("/");
            let vec = split_url.collect::<Vec<&str>>();
            let pull_url = format!(
                "https://github.com/{}/{}/pull/{}",
                vec[vec.len() - 4],
                vec[vec.len() - 3],
                vec[vec.len() - 1]
            );
            let reason = &reason
                .split("_")
                .collect::<Vec<&str>>().join(" ");
            notify("New Github Notification", reason, title, "Glass", &pull_url).await;
        }
    }
    if new_ids.len() > 1 {
        let ids_to_write: String = new_ids.iter().map( |&id| id.to_string() + ",").collect();
        fs::write(&ids_file_path, ids_to_write).expect("Unable to write ids to file");
    }
    if new_ids.len() == 1 {
        fs::write(&ids_file_path, new_ids[0]).expect("Unable to write ids to file");
    }
    Ok(())
}

async fn notify(title: &str, subtitle: &str, message: &str, sound: &str, open: &str) {
    let command: Output;
    if cfg!(target_os = "linux") {
        // the notify-send api does not permit on click actions, `open` and `sound` are unused
        let notification_str = format!("\"{title} ({subtitle})\" \"{message}\"");
        command = Command::new("sh")
            .arg("-c")
            .arg(format!("notify-send {notification_str}"))
            .output()
            .expect("failed to execute notify-send process");
    } else {
        let notification_str = format!("-title \"{title}\" -subtitle \"{subtitle}\" -message \"{message}\" -sound \"{sound}\" -open \"{open}\"");
        command = Command::new("sh")
            .arg("-c")
            .arg(format!("terminal-notifier {notification_str}"))
            .output()
            .expect("failed to execute terminal-notifier process");
    }
    let err = command.stderr;
    if err.len() > 0 {
        let err_disp = String::from_utf8(err)
            .expect("Could not decode error message (notify shell command) line 92");
        panic!("{}", err_disp)
    }
}

async fn error(error: &str) {
    notify(
        "GitHub Notifier",
        "error",
        error,
        "Pop",
        ""
    ).await
}

async fn connection_error(detail: &str) {
    let error_text: String = format!("Error connecting to GitHub API: {}", detail);
    error(&error_text).await
}
