use std::env;

pub fn get_token() -> Result<String, env::VarError> {
    let token_name = "GH_NOTIFIER_TOKEN";
    Ok(env::var(token_name)?)
}
