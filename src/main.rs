use std::env;
use std::process::Command;
use serde::Deserialize;
use reqwest::{Client, Error};
use reqwest::header::{AUTHORIZATION, ACCEPT, USER_AGENT};


#[derive(Deserialize, Debug)]
struct PR {
    title: String,
    url: String,
}

#[derive(Deserialize, Debug)]
struct Notification {
    subject: PR,
    reason: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new();
    let request_url = "https://api.github.com/notifications";
    let token = match env::var("GH_NOTIFIER_TOKEN") {
        Ok(t) => t,
        Err(e) => panic!("`GH_NOTIFIER_TOKEN` {}", e)
    };
    let response = client.get(request_url)
        .header(USER_AGENT, "Rust Reqwest")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(ACCEPT, "application/vnd.github+json")
        .send().await?;
    if response.status() != 200 {
        panic!("Error connecting to GitHub API: {}", response.status())
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
        let reason = &reason.split("_")
            .collect::<Vec<&str>>().join(" ");
        let notify_command = format!(
            "terminal-notifier -title \"New Github Notification\" \
            -subtitle \"{}\" -message \"{}\" -open {} -sound Glass",
            reason,
            title,
            pull_url
        );
        Command::new("sh")
            .arg("-c")
            .arg(&notify_command)
            .output()
            .expect("failed to execute terminal-notifier process");
    }
    Ok(())
}
