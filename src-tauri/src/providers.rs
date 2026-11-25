use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub provider: &'static str,
    pub id: &'static str,
    pub can_edit: bool,
}
pub struct Output {
    pub images: Vec<Vec<u8>>,
    pub usage: Option<serde_json::Value>,
}

pub struct Failure {
    pub message: String,
    pub uncertain: bool,
}
impl Failure {
    pub fn known(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            uncertain: false,
        }
    }
}
pub trait Provider {
    fn model(&self) -> Model;
    fn generate(&self, prompt: &str) -> Result<Output, Failure>;
}

#[cfg(any(test, feature = "fixture-provider"))]
#[derive(Default)]
pub struct Fixture {
    pub calls: std::cell::Cell<usize>,
}
#[cfg(any(test, feature = "fixture-provider"))]
impl Provider for Fixture {
    fn model(&self) -> Model {
        Model {
            provider: "fixture",
            id: "test-image",
            can_edit: false,
        }
    }
    fn generate(&self, _: &str) -> Result<Output, Failure> {
        self.calls.set(self.calls.get() + 1);
        Ok(Output {
            images: vec![include_bytes!("../icons/icon.png").to_vec()],
            usage: None,
        })
    }
}
