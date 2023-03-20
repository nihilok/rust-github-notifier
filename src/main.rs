mod cli;
mod errors;
mod file_operations;
mod notifier;
mod request;
mod token;

#[tokio::main]
async fn main() -> Result<(), errors::RuntimeErrors> {
    if cli::get_args() {
        return Ok(());
    }
    let gh_token = token::get_token()?;
    let notifications_json = match request::notifications_json(&gh_token).await {
        Ok(json) => json,
        Err(e) => return Err(errors::default_handler(e)?),
    };
    match notifier::notify_all(notifications_json) {
        Ok(()) => (),
        Err(e) => return Err(errors::default_handler(e)?),
    }
    Ok(())
}
