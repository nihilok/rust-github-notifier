use gh_notifier::*;

#[tokio::main]
async fn main() -> Result<(), errors::RuntimeErrors> {
    if cli::get_args() {
        return Ok(());
    }
    let gh_token = token::get_token()?;
    let notifications_json = match request::notifications_json(&gh_token).await {
        Ok(json) => json,
        Err(e) => return Err(errors::notify_and_return_error(e)?),
    };
    match notifier::notify_all(notifications_json) {
        Ok(()) => (),
        Err(e) => return Err(errors::notify_and_return_error(e)?),
    }
    Ok(())
}
