use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    #[default]
    Png,
    Jpeg,
    Webp,
    Svg,
    Gif,
}

impl Format {
    pub fn reference(bytes: &[u8]) -> Result<Self> {
        ensure!(
            bytes.len() < 50_000_000,
            "Choose a reference image smaller than 50 MB."
        );
        let format = match image::guess_format(bytes).ok() {
            Some(image::ImageFormat::Png) => Self::Png,
            Some(image::ImageFormat::Jpeg) => Self::Jpeg,
            Some(image::ImageFormat::WebP) => Self::Webp,
            _ => anyhow::bail!("Choose a PNG, JPEG or WebP reference image."),
        };
        format
            .validate(bytes)
            .context("This image could not be read. Choose another image.")?;
        Ok(format)
    }
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Webp => "webp",
            Self::Svg => "svg",
            Self::Gif => "gif",
        }
    }
    pub fn mime(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
            Self::Svg => "image/svg+xml",
            Self::Gif => "image/gif",
        }
    }
    pub fn parse(ext: &str) -> Option<Self> {
        [Self::Png, Self::Jpeg, Self::Webp, Self::Svg, Self::Gif]
            .into_iter()
            .find(|f| f.extension() == ext)
    }
    pub fn editable(self) -> bool {
        matches!(self, Self::Png | Self::Webp | Self::Jpeg)
    }
    pub fn validate(self, bytes: &[u8]) -> Result<()> {
        let raster = match self {
            Self::Png => image::ImageFormat::Png,
            Self::Jpeg => image::ImageFormat::Jpeg,
            Self::Webp => image::ImageFormat::WebP,
            Self::Gif => image::ImageFormat::Gif,
            Self::Svg => {
                let xml = std::str::from_utf8(bytes).context("Invalid SVG text.")?;
                let doc = roxmltree::Document::parse(xml).context("Invalid SVG document.")?;
                ensure!(
                    doc.root_element()
                        .has_tag_name(("http://www.w3.org/2000/svg", "svg")),
                    "Expected an SVG document."
                );
                return Ok(());
            }
        };
        ensure!(
            image::guess_format(bytes)? == raster,
            "The returned image format does not match the request."
        );
        image::load_from_memory_with_format(bytes, raster).context("Unreadable image data.")?;
        Ok(())
    }
}

pub struct Original {
    pub bytes: Vec<u8>,
    pub format: Format,
}
