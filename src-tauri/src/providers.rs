#[cfg(any(test, not(feature = "fixture-provider")))]
pub mod openai;
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
    fn render(&self, prompt: &str, original: Option<&[u8]>) -> Result<Output, Failure>;
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
            can_edit: true,
        }
    }
    fn render(&self, _: &str, original: Option<&[u8]>) -> Result<Output, Failure> {
        self.calls.set(self.calls.get() + 1);
        let images = if let Some(bytes) = original {
            vec![bytes.to_vec()]
        } else {
            let first = include_bytes!("../icons/icon.png");
            let flipped = image::load_from_memory(first).expect("fixture PNG").fliph();
            let mut second = std::io::Cursor::new(Vec::new());
            flipped
                .write_to(&mut second, image::ImageFormat::Png)
                .expect("fixture PNG");
            vec![first.to_vec(), second.into_inner()]
        };
        Ok(Output {
            images,
            usage: None,
        })
    }
}
