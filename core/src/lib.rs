pub mod catalog;
pub mod jobs;
pub mod library;
pub mod media;
pub mod providers;

#[cfg(any(test, feature = "fixture-provider"))]
const FIXTURE_PNG: &[u8] = include_bytes!("../../src-tauri/icons/icon.png");
