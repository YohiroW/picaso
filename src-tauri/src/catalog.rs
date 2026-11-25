use crate::{jobs::ImageOptions, media::Format};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, io::ErrorKind, path::PathBuf};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Model {
    pub id: String,
    pub display_name: String,
    pub provider: String,
    pub can_edit: bool,
    pub sizes: Vec<String>,
    pub custom_size: bool,
    pub qualities: Vec<String>,
    pub formats: Vec<Format>,
    pub backgrounds: Vec<String>,
    pub max_images: u8,
    pub defaults: ImageOptions,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File<T> {
    models: Vec<T>,
}

pub fn bundled() -> Vec<Model> {
    serde_json::from_str::<File<Model>>(include_str!("../models.json"))
        .expect("bundled models.json")
        .models
}

pub struct Catalog(pub PathBuf);
impl Catalog {
    pub fn models(&self) -> Result<Vec<Model>> {
        let mut models = bundled();
        let data = match fs::read(&self.0) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(models),
            Err(e) => return Err(e).context("Could not read models.json"),
        };
        let file: File<serde_json::Value> =
            serde_json::from_slice(&data).context("Invalid models.json")?;
        let mut ids = HashSet::new();
        for patch in file.models {
            let id = patch
                .get("id")
                .and_then(|v| v.as_str())
                .context("Each models.json entry needs an id.")?;
            let mut value = models
                .iter()
                .find(|m| m.id == id)
                .map(serde_json::to_value)
                .transpose()?
                .unwrap_or(serde_json::Value::Null);
            merge(&mut value, patch);
            let entry: Model = serde_json::from_value(value).context("Invalid model definition")?;
            ensure!(
                entry.provider == "openai"
                    && !entry.id.trim().is_empty()
                    && !entry.display_name.trim().is_empty(),
                "Invalid model ID, name or provider in models.json."
            );
            ensure!(
                ids.insert(entry.id.clone()),
                "Duplicate model ID in models.json: {}",
                entry.id
            );
            ensure!(
                (1..=4).contains(&entry.max_images),
                "maxImages must be between 1 and 4."
            );
            ensure!(
                entry.qualities.iter().all(|s| matches!(
                    s.as_str(),
                    "auto" | "low" | "medium" | "high" | "xhigh" | "max"
                )) && entry
                    .backgrounds
                    .iter()
                    .all(|s| matches!(s.as_str(), "auto" | "opaque" | "transparent")),
                "Unknown quality or background in models.json."
            );
            ensure!(
                entry
                    .sizes
                    .iter()
                    .all(|s| s == "auto" || crate::jobs::valid_dimensions(s)),
                "Invalid size preset in models.json."
            );
            entry
                .defaults
                .validate(&entry, false)
                .with_context(|| format!("Invalid defaults for {}", entry.id))?;
            if let Some(old) = models.iter_mut().find(|m| m.id == entry.id) {
                *old = entry;
            } else {
                models.push(entry);
            }
        }
        Ok(models)
    }
}
fn merge(base: &mut serde_json::Value, patch: serde_json::Value) {
    match (base, patch) {
        (serde_json::Value::Object(base), serde_json::Value::Object(patch)) => {
            for (key, value) in patch {
                merge(base.entry(key).or_insert(serde_json::Value::Null), value);
            }
        }
        (target, value) => *target = value,
    }
}

#[cfg(any(test, feature = "fixture-provider"))]
pub fn fixture() -> Model {
    let mut model = bundled().into_iter().next().expect("fixture capabilities");
    model.id = "test-image".into();
    model.display_name = "Local test images".into();
    model.provider = "fixture".into();
    model.defaults = ImageOptions::default();
    model
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overrides_defaults_adds_models_and_reports_broken_configuration() {
        let root = tempfile::tempdir().unwrap();
        let catalog = Catalog(root.path().join("models.json"));
        let base = catalog.models().unwrap();
        let mut extra = base[0].clone();
        extra.id = "test-custom".into();
        let file = serde_json::json!({"models":[{"id":base[1].id,"defaults":{"background":"transparent"}},extra]});
        fs::write(&catalog.0, file.to_string()).unwrap();
        let models = catalog.models().unwrap();
        assert_eq!(models.len(), base.len() + 1);
        assert_eq!(models[1].defaults.background, "transparent");
        assert_eq!(models[1].qualities, base[1].qualities);
        assert_eq!(models.last().unwrap().id, "test-custom");
        fs::write(
            &catalog.0,
            r#"{"models":[{"id":"gpt-image-2","maxImages":5}]}"#,
        )
        .unwrap();
        assert!(catalog.models().is_err());
    }
}
