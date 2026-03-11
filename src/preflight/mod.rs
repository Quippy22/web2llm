mod robots;
mod validate;

use url::Url;

use crate::{Result, Web2llmError};

pub(crate) async fn run(raw_url: &str, user_agent: &str, block_private_hosts: bool) -> Result<Url> {
    let url = validate::validate(raw_url, block_private_hosts)?;

    if !robots::is_allowed(&url, user_agent).await? {
        return Err(Web2llmError::Disallowed);
    }

    Ok(url)
}
