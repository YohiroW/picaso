pub mod openai;
pub use crate::catalog::Model;
use crate::jobs::ImageOptions;
use crate::media::Original;
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
    fn render(
        &self,
        prompt: &str,
        original: Option<&Original>,
        options: &ImageOptions,
    ) -> Result<Output, Failure>;
}

#[cfg(any(test, feature = "fixture-provider"))]
#[derive(Default)]
pub struct Fixture {
    pub calls: std::cell::Cell<usize>,
}
#[cfg(any(test, feature = "fixture-provider"))]
impl Provider for Fixture {
    fn model(&self) -> Model {
        crate::catalog::fixture()
    }
    fn render(
        &self,
        _: &str,
        original: Option<&Original>,
        options: &ImageOptions,
    ) -> Result<Output, Failure> {
        use crate::media::Format;
        self.calls.set(self.calls.get() + 1);
        let input = original
            .map(|im| im.bytes.as_slice())
            .unwrap_or(crate::FIXTURE_PNG);
        let mut images = Vec::new();
        for i in 0..options.n {
            if options.output_format == Format::Svg {
                images.push(br#"<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128"><circle cx="64" cy="64" r="50" fill="coral"/></svg>"#.to_vec());
                continue;
            }
            let format = match options.output_format {
                Format::Png => image::ImageFormat::Png,
                Format::Jpeg => image::ImageFormat::Jpeg,
                Format::Webp => image::ImageFormat::WebP,
                Format::Gif => image::ImageFormat::Gif,
                Format::Svg => unreachable!(),
            };
            if i == 0 && image::guess_format(input).ok() == Some(format) {
                images.push(input.to_vec());
                continue;
            }
            let mut image = image::load_from_memory(input)
                .map_err(|_| Failure::known("Unreadable fixture input."))?;
            if i % 2 == 1 {
                image = image.fliph();
            }
            if format == image::ImageFormat::Jpeg {
                image = image::DynamicImage::ImageRgb8(image.to_rgb8());
            }
            let mut buffer = std::io::Cursor::new(Vec::new());
            image
                .write_to(&mut buffer, format)
                .map_err(|_| Failure::known("Could not encode fixture image."))?;
            images.push(buffer.into_inner());
        }
        Ok(Output {
            images,
            usage: None,
        })
    }
}
