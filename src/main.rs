use std::{env, process};
use std::process::Command;
use serde::Deserialize;
use reqwest::{Client, Error};
use reqwest::header::{AUTHORIZATION, ACCEPT, USER_AGENT};

#[derive(Deserialize, Debug)]
struct NotificationSubject {
    title: String,
    url: String,
}

#[derive(Deserialize, Debug)]
struct Notification {
    subject: NotificationSubject,
    reason: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new();
    let request_url = "https://api.github.com/notifications";
    let env_var_name = "GH_NOTIFIER_TOKEN";
    let token = match env::var(env_var_name) {
        Ok(t) => t,
        Err(e) => {
            let error_text = format!("{} {}", env_var_name, e);
            error(&error_text).await;
            println!("{}", error_text);
            process::exit(1);
        }
    };
    let response = match client.get(request_url)
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
    for notification in &response_json {
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
        let notification_str = format!(
            "-title \"New Github Notification\" \
            -subtitle \"{}\" -message \"{}\" -open {} \
            -sound Glass",
            reason,
            title,
            pull_url
        );
        notify(&notification_str).await;
    }
    Ok(())
}

async fn notify(notification_str: &str) {
    Command::new("sh")
        .arg("-c")
        .arg(format!("terminal-notifier {notification_str}"))
        .output()
        .expect("failed to execute terminal-notifier process");
}

async fn error(error: &str) {
    let notification_str = format!(
        "-title \"Github Notifier\" \
        -subtitle \"error\" -message \"{}\" \
        -sound Pop",
        error,
    );
    notify(&notification_str).await;
}

async fn connection_error(detail: &str) {
    let error_text = format!("Error connecting to GitHub API: {}", detail);
    error(&error_text).await;
}
