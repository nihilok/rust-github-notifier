use std::{env, process};

use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Client, Error, StatusCode};
use serde::Deserialize;

pub mod util;

use crate::util::{get_local_ids, parse_args, save_local_ids};
use util::{
    build_pull_or_issue_url, display_new_github_notification, get_persistence_file_path,
    notify_error,
};

const REQUEST_URL: &str = "https://api.github.com/notifications";
const ENV_VAR_NAME: &str = "GH_NOTIFIER_TOKEN";

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    if parse_args() {
        return Ok(());
    }
    // if no cli arguments used, proceed with main actions:

    // get token from environment variable
    let token = match env::var(ENV_VAR_NAME) {
        Ok(t) => t,
        Err(err) => {
            let error_text = format!("{} {}", ENV_VAR_NAME, err);
            notify_error(&error_text);
            dbg!(error_text);
            process::exit(1);
        }
    };

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
        Ok(response) => response,
        Err(err) => {
            notify_error(&format!("{err}"));
            dbg!(err);
            process::exit(1);
        }
    };

    // handle unsuccessful responses
    let status = response.status();
    if status != StatusCode::OK {
        let text = response.text().await?;
        let error_text: String = format!("Response: {} {}", status, text);
        notify_error(&error_text);
        dbg!(error_text);
        process::exit(1);
    };

    // handle successful API response
    let response_json: Vec<Notification> = response.json().await?;

    // read/parse already notified ids from file
    let fs_path = get_persistence_file_path();
    let read_ids_str = get_local_ids(&fs_path);
    let mut new_ids: Vec<String> = Vec::new();

    // loop through notifications in response, checking against saved notification ids
    // and display desktop notification if identifier not already saved to file
    for notification in &response_json {
        let mut identifier: String = notification.id.to_owned();
        identifier.push_str(&notification.updated_at);
        let check = identifier.clone();
        new_ids.push(identifier);
        if read_ids_str.contains(&check.as_str()) {
            // have already notified about this notification
            continue;
        }

        // build notification parts
        let message = &notification.subject.title;
        let optional_url = notification.subject.url.clone();
        let onclick_url = build_pull_or_issue_url(optional_url);
        let reason_vec = &notification.reason.split("_").collect::<Vec<&str>>();
        let subtitle = reason_vec.join(" ");

        display_new_github_notification(message, onclick_url.as_str(), subtitle.as_str());
    }

    save_local_ids(new_ids, &fs_path);
    Ok(())
}
