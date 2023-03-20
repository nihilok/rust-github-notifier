use crate::errors::RuntimeErrors;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NotificationSubject {
    pub title: String,
    pub url: Option<String>,
}

#[derive(Deserialize)]
pub struct Notification {
    pub id: String,
    pub subject: NotificationSubject,
    pub reason: String,
    pub updated_at: String,
}

pub async fn notifications_json(token: &str) -> Result<Vec<Notification>, RuntimeErrors> {
    let url = "https://api.github.com/notifications";
    let client = Client::new();
    let response = client
        .get(url)
        .header(USER_AGENT, "Rust Reqwest")
        .header(AUTHORIZATION, format!("Bearer {token}"))
        .header(ACCEPT, "application/vnd.github+json")
        .send()
        .await?;
    response.error_for_status_ref()?;
    let response_json: Vec<Notification> = response.json().await?;
    Ok(response_json)
}
