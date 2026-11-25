use crate::{library::Library, providers::Provider};
use anyhow::{bail, Result};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub request_id: Uuid,
    pub prompt: String,
}
impl Submission {
    #[cfg(test)]
    pub fn new(prompt: &str) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            prompt: prompt.into(),
        }
    }
}
pub const SIZE: &str = "1024x1024";
pub const QUALITY: &str = "low";

pub fn submit(lib: &Library, provider: &dyn Provider, draft: &Submission) -> Result<String> {
    let prompt = draft.prompt.trim();
    if prompt.is_empty() {
        bail!("Write a prompt before submitting.");
    }
    let params = serde_json::json!({"size": SIZE, "quality": QUALITY, "output_format":"png","n":1});
    if !lib.reserve(
        &draft.request_id,
        prompt,
        provider.model(),
        None,
        &params.to_string(),
    )? {
        return Ok(draft.request_id.to_string());
    }
    match provider.generate(prompt) {
        Ok(output) => {
            if let Err(e) = lib.finish(&draft.request_id, output) {
                let message = format!("Couldn’t save the result: {e}. Check your data folder and free space. Submitting again may incur another charge.");
                lib.fail(&draft.request_id, &message, false)?;
                bail!("{message}")
            }
        }
        Err(e) => {
            lib.fail(&draft.request_id, &e.message, e.uncertain)?;
            bail!("{}", e.message);
        }
    }
    Ok(draft.request_id.to_string())
}

#[cfg(test)]
mod tests;
