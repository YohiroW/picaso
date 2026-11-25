use serde::{Deserialize, Serialize};
use std::{fs, io::ErrorKind, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Environment,
    Keychain,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub selected: Option<Source>,
    pub environment: bool,
    pub keychain: bool,
    pub keychain_error: bool,
    pub preference_error: bool,
}

pub trait Vault {
    fn get(&self) -> Result<Option<String>, String>;
    fn set(&self, value: &str) -> Result<(), String>;
    fn remove(&self) -> Result<(), String>;
}

pub struct MacKeychain(pub String);
const SERVICE: &str = "app.picaso.desktop";

const KEYCHAIN_ERROR: &str =
    "Couldn’t access the saved key. Unlock your login keychain and try again.";

#[cfg(target_os = "macos")]
impl Vault for MacKeychain {
    fn get(&self) -> Result<Option<String>, String> {
        match security_framework::passwords::get_generic_password(SERVICE, &self.0) {
            Ok(bytes) => String::from_utf8(bytes)
                .map(Some)
                .map_err(|_| "Replace the unreadable saved key.".into()),
            Err(error) if error.code() == -25300 => Ok(None),
            Err(_) => Err(KEYCHAIN_ERROR.into()),
        }
    }
    fn set(&self, value: &str) -> Result<(), String> {
        security_framework::passwords::set_generic_password(SERVICE, &self.0, value.as_bytes())
            .map_err(|_| KEYCHAIN_ERROR.into())
    }
    fn remove(&self) -> Result<(), String> {
        match security_framework::passwords::delete_generic_password(SERVICE, &self.0) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == -25300 => Ok(()),
            Err(_) => Err(KEYCHAIN_ERROR.into()),
        }
    }
}
#[cfg(not(target_os = "macos"))]
impl Vault for MacKeychain {
    fn get(&self) -> Result<Option<String>, String> {
        Ok(None)
    }
    fn set(&self, _: &str) -> Result<(), String> {
        Err("Keychain storage requires macOS.".into())
    }
    fn remove(&self) -> Result<(), String> {
        self.set("")
    }
}

pub struct Credentials<V> {
    root: PathBuf,
    vault: V,
    environment: Option<String>,
}

impl<V: Vault> Credentials<V> {
    pub fn new(root: PathBuf, vault: V, environment: Option<String>) -> Self {
        Self {
            root,
            vault,
            environment: environment.filter(|s| !s.trim().is_empty()),
        }
    }
    fn selected(&self) -> Result<Option<Source>, String> {
        match fs::read(self.root.join("credential-source.json")) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|_| "Couldn’t read your key preference. Choose a source again.".into()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(_) => Err("Couldn’t read your key preference.".into()),
        }
    }
    fn remember(&self, source: Source) -> Result<Status, String> {
        let mut tmp = tempfile::NamedTempFile::new_in(&self.root).map_err(|e| e.to_string())?;
        serde_json::to_writer(&mut tmp, &source).map_err(|e| e.to_string())?;
        tmp.persist(self.root.join("credential-source.json"))
            .map_err(|e| e.error.to_string())?;
        self.status()
    }
    pub fn status(&self) -> Result<Status, String> {
        let key = self.vault.get();
        let source = self.selected();
        Ok(Status {
            preference_error: source.is_err(),
            selected: source.ok().flatten(),
            environment: self.environment.is_some(),
            keychain: matches!(&key, Ok(Some(_))),
            keychain_error: key.is_err(),
        })
    }
    fn read_source(&self, source: Source) -> Result<String, String> {
        let key = match source {
            Source::Environment => self.environment.clone(),
            Source::Keychain => self.vault.get()?,
        };
        key.ok_or_else(|| "Your selected key is unavailable. Open Key settings to restore it or choose another source.".into())
    }
    #[cfg(any(test, not(feature = "fixture-provider")))]
    pub fn resolve(&self) -> Result<String, String> {
        self.read_source(
            self.selected()?
                .ok_or("Choose a key in Key settings first.")?,
        )
    }
    pub fn select(&self, source: Source) -> Result<Status, String> {
        self.read_source(source)?;
        self.remember(source)
    }
    pub fn save(&self, key: &str) -> Result<Status, String> {
        let key = key.trim();
        if key.is_empty() || !key.bytes().all(|b| b.is_ascii_graphic()) {
            return Err("Enter an API key without spaces or line breaks.".into());
        }
        self.vault.set(key)?;
        self.remember(Source::Keychain)
    }
    pub fn remove(&self) -> Result<Status, String> {
        self.vault.remove()?;
        self.status()
    }
}

#[cfg(test)]
mod tests;
