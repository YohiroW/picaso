use super::{Failure, Model, Output, Provider};
use crate::media::Original;
use crate::{jobs::ImageOptions, library::UNCERTAIN};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::blocking::{multipart, Client};
use serde::Deserialize;
use std::time::Duration;

const API: &str = "https://api.openai.com/v1/images";

pub struct OpenAi {
    key: String,
    model: Model,
    client: Client,
    #[cfg(test)]
    test_origin: Option<String>,
}

impl OpenAi {
    pub fn new(key: String, model: Model) -> Result<Self, String> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(20))
            .timeout(Duration::from_secs(300))
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .build()
            .map_err(|_| "Couldn’t initialize the network client.")?;
        Ok(Self {
            key,
            model,
            client,
            #[cfg(test)]
            test_origin: None,
        })
    }
}

#[derive(Deserialize)]
struct Images {
    data: Vec<ImageData>,
    usage: Option<serde_json::Value>,
}
#[derive(Deserialize)]
struct ImageData {
    b64_json: String,
}

impl Provider for OpenAi {
    fn model(&self) -> Model {
        self.model.clone()
    }
    fn render(
        &self,
        prompt: &str,
        original: Option<&Original>,
        options: &ImageOptions,
    ) -> Result<Output, Failure> {
        let api = API;
        #[cfg(test)]
        let api = self.test_origin.as_deref().unwrap_or(api);
        let mut fields = serde_json::to_value(options).expect("serializable image options");
        fields["model"] = serde_json::json!(self.model.id);
        fields["prompt"] = serde_json::json!(prompt);
        let request = match original {
            Some(original) => {
                let image = multipart::Part::bytes(original.bytes.clone())
                    .file_name(format!("original.{}", original.format.extension()))
                    .mime_str(original.format.mime())
                    .expect("known image MIME type");
                let mut form = multipart::Form::new().part("image", image);
                for (name, value) in fields.as_object().expect("image request object") {
                    let text = value
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| value.to_string());
                    form = form.text(name.clone(), text);
                }
                self.client.post(format!("{api}/edits")).multipart(form)
            }
            None => self.client.post(format!("{api}/generations")).json(&fields),
        };
        let response = request.bearer_auth(&self.key).send().map_err(|e| {
            if e.is_connect() {
                Failure::known(
                    "Couldn’t connect to OpenAI. Check your connection and proxy settings.",
                )
            } else {
                uncertain("The connection ended before the result arrived.")
            }
        })?;
        let status = response.status();
        if !status.is_success() {
            let body = response.json::<serde_json::Value>().ok();
            let code = body
                .as_ref()
                .and_then(|v| v.pointer("/error/code"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            return Err(api_failure(status.as_u16(), code));
        }
        let result = response
            .json::<Images>()
            .map_err(|_| uncertain("OpenAI returned an unreadable result."))?;
        if result.data.is_empty() {
            return Err(uncertain("OpenAI returned no image."));
        }
        let images = result
            .data
            .into_iter()
            .map(|image| {
                STANDARD
                    .decode(image.b64_json)
                    .map_err(|_| uncertain("OpenAI returned unreadable image data."))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Output {
            images,
            usage: result.usage,
        })
    }
}
fn uncertain(reason: &str) -> Failure {
    Failure {
        message: format!("{reason} {UNCERTAIN}"),
        uncertain: true,
    }
}
fn api_failure(status: u16, code: &str) -> Failure {
    if status >= 500 || status == 408 {
        return uncertain("OpenAI could not confirm completion.");
    }
    Failure::known(match (status,code) {
        (401,_) => "OpenAI rejected this API key. Replace it or choose another key in Key settings.",
        (403,_) | (_, "model_not_found") => "This account cannot use the image model. Check model access and organization verification.",
        (402,_) | (_, "insufficient_quota" | "billing_hard_limit_reached" | "billing_not_active") => "OpenAI API credit is unavailable. Check billing, balance and spending limits.",
        (429,_) => "OpenAI’s rate limit was reached. Wait before submitting again.",
        (_, "content_policy_violation" | "moderation_blocked") => "OpenAI declined this image request. Revise your prompt or reference.",
        (400,_) => "OpenAI could not accept this request. Check the size, count and output settings, or revise the prompt or reference.",
        _ => "OpenAI rejected the request. Check your API account and model access.",
    })
}

#[cfg(test)]
mod tests;
