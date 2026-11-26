use clap::{Args, Parser, Subcommand};
use picaso_core::{jobs::ImageOptions, media::Format};
use std::{
    io::{self, IsTerminal, Read},
    path::PathBuf,
};
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "picaso",
    version,
    about = "Generate, edit and export images from the command line."
)]
pub struct Cli {
    /// Persistent library directory for this workflow.
    #[arg(long, global = true)]
    pub workspace: Option<PathBuf>,
    /// Write compact JSON (the default is pretty-printed JSON).
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// List configured models, capabilities and defaults.
    Models,
    /// Generate images from a prompt.
    Generate(SubmitArgs),
    /// Edit one original from this workspace.
    Edit {
        #[arg(long)]
        image_id: Uuid,
        #[command(flatten)]
        args: SubmitArgs,
    },
    /// Look up a saved job and its available originals.
    Show { job_id: Uuid },
    /// Copy an original to a file outside the workspace.
    Export {
        #[arg(long)]
        image_id: Uuid,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Args)]
pub struct SubmitArgs {
    /// Inline prompt. With neither prompt option, read piped stdin.
    #[arg(long, conflicts_with = "prompt_file")]
    prompt: Option<String>,
    /// UTF-8 text file, or - to read all of stdin.
    #[arg(long)]
    prompt_file: Option<PathBuf>,
    /// Reuse this UUID when replaying a workflow step.
    #[arg(long)]
    pub request_id: Option<Uuid>,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long)]
    size: Option<String>,
    #[arg(long)]
    quality: Option<String>,
    #[arg(long, alias = "n")]
    count: Option<u8>,
    #[arg(long, value_parser = parse_format)]
    output_format: Option<Format>,
    #[arg(long)]
    background: Option<String>,
    /// Copy outputs here. Otherwise paths refer to workspace originals.
    #[arg(long)]
    pub out_dir: Option<PathBuf>,
    /// Read OPENAI_API_KEY from the calling process.
    #[arg(long, value_parser = ["env"], default_value = "env")]
    key_source: String,
}

fn parse_format(value: &str) -> Result<Format, String> {
    Format::parse(value).ok_or_else(|| "Expected png, jpeg, webp, svg or gif.".into())
}

impl SubmitArgs {
    pub fn read_prompt(&self) -> anyhow::Result<String> {
        let text = if let Some(prompt) = &self.prompt {
            prompt.clone()
        } else if let Some(path) = self
            .prompt_file
            .as_deref()
            .filter(|p| *p != std::path::Path::new("-"))
        {
            std::fs::read_to_string(path)?
        } else {
            let stdin = io::stdin();
            anyhow::ensure!(
                self.prompt_file.is_some() || !stdin.is_terminal(),
                "Pipe a prompt to stdin, or specify --prompt or --prompt-file."
            );
            let mut text = String::new();
            stdin.lock().read_to_string(&mut text)?;
            text
        };
        anyhow::ensure!(!text.trim().is_empty(), "The prompt is empty.");
        Ok(text)
    }

    pub fn options(&self, defaults: ImageOptions) -> ImageOptions {
        ImageOptions {
            size: self.size.clone().unwrap_or(defaults.size),
            quality: self.quality.clone().unwrap_or(defaults.quality),
            n: self.count.unwrap_or(defaults.n),
            output_format: self.output_format.unwrap_or(defaults.output_format),
            background: self.background.clone().unwrap_or(defaults.background),
        }
    }
}
