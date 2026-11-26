use crate::{catalog::Model, media::Format};
use crate::{library::Library, providers::Provider};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum SubmissionError {
    #[error("This request ID was already used for a different submission.")]
    Conflict,
    #[error("{0}")]
    Unconfirmed(String),
    #[error("{0}")]
    Failed(String),
    #[error("This request is already running.")]
    Running,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub request_id: Uuid,
    pub prompt: String,
    pub model: Option<String>,
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
            model: None,
            options: ImageOptions::default(),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ImageOptions {
    pub size: String,
    pub quality: String,
    pub n: u8,
    pub output_format: Format,
    pub background: String,
}
impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            size: "1024x1024".into(),
            quality: "low".into(),
            n: 1,
            output_format: Format::Png,
            background: "auto".into(),
        }
    }
}
pub fn valid_dimensions(size: &str) -> bool {
    let Some((w, h)) = size.split_once('x') else {
        return false;
    };
    let (Ok(w), Ok(h)) = (w.parse::<u32>(), h.parse::<u32>()) else {
        return false;
    };
    w > 0
        && h > 0
        && w <= 3840
        && h <= 3840
        && w % 16 == 0
        && h % 16 == 0
        && w.max(h) <= 3 * w.min(h)
        && (655360..=8294400).contains(&(w * h))
        && size == format!("{w}x{h}")
}
impl ImageOptions {
    pub fn validate(&self, model: &Model, editing: bool) -> Result<()> {
        if !model.sizes.contains(&self.size) && !(model.custom_size && valid_dimensions(&self.size))
        {
            bail!("Choose a supported size. Custom edges must be multiples of 16, up to 3840 px, within 3:1 and 655,360–8,294,400 pixels.");
        }
        if !model.qualities.contains(&self.quality) {
            bail!("Choose a quality supported by this model.");
        }
        if !(1..=model.max_images.min(4)).contains(&self.n) {
            bail!("Choose an available image count.");
        }
        if !model.formats.contains(&self.output_format) {
            bail!("Choose an output format supported by this model.");
        }
        if !model.backgrounds.contains(&self.background) {
            bail!("Choose a background supported by this model.");
        }
        if self.background == "transparent"
            && !matches!(self.output_format, Format::Png | Format::Webp)
        {
            bail!("Transparent backgrounds require PNG or WebP.");
        }
        if self.output_format == Format::Svg && (editing || self.n != 1) {
            bail!("SVG requires a new generation with one image.");
        }
        Ok(())
    }
}

pub fn submit(lib: &Library, provider: &dyn Provider, draft: &Submission) -> Result<String> {
    let prompt = draft.prompt.trim();
    if prompt.is_empty() {
        bail!("Write a prompt before submitting.");
    }
    let model = provider.model();
    if draft.model.as_ref().is_some_and(|id| id != &model.id) {
        bail!("The selected model is unavailable.");
    }
    draft.options.validate(&model, draft.input_id.is_some())?;
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
    if let Some(image) = &original {
        if !image.format.editable() {
            bail!("Only PNG, JPEG and WebP originals can be used as edit inputs.");
        }
        image.format.validate(&image.bytes).context("The original image is damaged or has the wrong format. Restore it or choose another image.")?;
    }
    if !lib.reserve(draft, &model)? {
        return Ok(draft.request_id.to_string());
    }
    match provider.render(prompt, original.as_ref(), &draft.options) {
        Ok(output) => {
            if let Err(e) = lib.finish(&draft.request_id, output, draft.options.output_format) {
                let message = format!("Couldn’t save the result: {e}. Check your data folder and free space. Submitting again may incur another charge.");
                lib.fail(&draft.request_id, &message, false)?;
                return Err(SubmissionError::Failed(message).into());
            }
        }
        Err(e) => {
            lib.fail(&draft.request_id, &e.message, e.uncertain)?;
            return Err(if e.uncertain {
                SubmissionError::Unconfirmed(e.message)
            } else {
                SubmissionError::Failed(e.message)
            }
            .into());
        }
    }
    Ok(draft.request_id.to_string())
}

#[cfg(test)]
mod tests;
