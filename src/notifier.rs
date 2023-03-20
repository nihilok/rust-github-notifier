use crate::errors::RuntimeErrors;
use crate::file_operations;
use crate::request;
use notify::NotificationBuilder;

pub fn notify_error(error: &str) -> Result<(), RuntimeErrors> {
    match NotificationBuilder::default()
        .title("Github Notifier")
        .subtitle("Error")
        .message(error)
        .sound("Pop")
        .build()
    {
        Ok(n) => {
            n.notify();
            Ok(())
        }
        Err(e) => {
            return Err(RuntimeErrors::Notification(e));
        }
    }
}

fn new_github_notification(
    message: &str,
    onclick_url: &str,
    subtitle: &str,
) -> Result<(), RuntimeErrors> {
    match NotificationBuilder::default()
        .title("New Github Notification")
        .subtitle(subtitle)
        .message(message)
        .open(onclick_url)
        .build()
    {
        Ok(n) => {
            n.notify();
            Ok(())
        }
        Err(e) => {
            return Err(RuntimeErrors::Notification(e));
        }
    }
}

fn build_pull_or_issue_url(url: &Option<String>) -> String {
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

pub fn notify_all(notifications_json: Vec<request::Notification>) -> Result<(), RuntimeErrors> {
    let path = file_operations::get_persistence_file_path(".gh-notifier-read-notifications");
    let read_ids = file_operations::get_local_ids(&path);
    let mut new_ids: Vec<String> = Vec::new();

    for notification in &notifications_json {
        let mut identifier: String = notification.id.to_owned();
        identifier.push_str(&notification.updated_at);
        new_ids.push(identifier.clone());
        if read_ids.contains(&identifier.as_str()) {
            continue;
        }

        let message = &notification.subject.title;
        let optional_url = &notification.subject.url;
        let onclick_url = build_pull_or_issue_url(optional_url);
        let reason_vec = &notification.reason.split("_").collect::<Vec<&str>>();
        let subtitle = reason_vec.join(" ");

        new_github_notification(message, onclick_url.as_str(), subtitle.as_str())?;
    }

    file_operations::save_local_ids(new_ids, &path)?;
    Ok(())
}
