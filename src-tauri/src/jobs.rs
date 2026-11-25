use crate::{library::Library, providers::Provider};
use anyhow::{bail, Result};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub request_id: Uuid,
    pub prompt: String,
    pub input_id: Option<String>,
    #[serde(default)]
    pub options: ImageOptions,
}
impl Submission {
    #[cfg(test)]
    pub fn new(prompt: &str) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            prompt: prompt.into(),
            input_id: None,
            options: ImageOptions::default(),
        }
    }
}
#[derive(Deserialize)]
pub struct ImageOptions {
    pub size: String,
    pub quality: String,
}
impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            size: "1024x1024".into(),
            quality: "low".into(),
        }
    }
}
impl ImageOptions {
    fn validate(&self) -> Result<()> {
        if !matches!(self.size.as_str(), "1024x1024" | "1536x1024" | "1024x1536") {
            bail!("Choose a supported image size.");
        }
        if !matches!(self.quality.as_str(), "low" | "medium" | "high") {
            bail!("Choose a supported quality.");
        }
        Ok(())
    }
}

pub fn submit(lib: &Library, provider: &dyn Provider, draft: &Submission) -> Result<String> {
    let prompt = draft.prompt.trim();
    if prompt.is_empty() {
        bail!("Write a prompt before submitting.");
    }
    draft.options.validate()?;
    let model = provider.model();
    if draft.input_id.is_some() && !model.can_edit {
        bail!(
            "This model cannot edit an existing image. Remove the reference to start a new image."
        );
    }
    let original = draft
        .input_id
        .as_deref()
        .map(|id| lib.original(id))
        .transpose()?;
    let params = serde_json::json!({"size": draft.options.size, "quality": draft.options.quality, "output_format":"png","n":1});
    if !lib.reserve(
        &draft.request_id,
        prompt,
        model,
        draft.input_id.as_deref(),
        &params.to_string(),
    )? {
        return Ok(draft.request_id.to_string());
    }
    match provider.render(prompt, original.as_deref(), &draft.options) {
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
