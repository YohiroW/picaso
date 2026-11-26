mod args;

use args::{Cli, Command, SubmitArgs};
use clap::{error::ErrorKind, Parser};
use picaso_core::{
    catalog::{Catalog, ModelSet},
    jobs::{self, Submission, SubmissionError},
    library::{Job, Library, WorkspaceBusy},
    media::Format,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    fmt::Display,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};
use uuid::Uuid;

#[cfg(not(feature = "fixture-provider"))]
const MODELS: ModelSet = ModelSet::OpenAi;
#[cfg(feature = "fixture-provider")]
const MODELS: ModelSet = ModelSet::Fixture;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageFile {
    image_id: String,
    format: Format,
    path: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CliError {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    job_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    images: Vec<ImageFile>,
}

type Result<T> = std::result::Result<T, CliError>;

impl CliError {
    fn new(code: &'static str, error: impl Display) -> Self {
        Self {
            code,
            message: error.to_string(),
            job_id: None,
            images: vec![],
        }
    }

    fn submission(error: anyhow::Error, id: Uuid) -> Self {
        let code = match error.downcast_ref::<SubmissionError>() {
            Some(SubmissionError::Conflict) => "request_conflict",
            Some(SubmissionError::Unconfirmed(_)) => "unconfirmed",
            Some(SubmissionError::Failed(_)) => "job_failed",
            Some(SubmissionError::Running) => "job_running",
            None => "submission_error",
        };
        Self {
            job_id: Some(id),
            ..Self::new(code, error)
        }
    }
}

fn workspace(path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = path {
        return Ok(path);
    }
    let home = std::env::home_dir().ok_or_else(|| {
        CliError::new(
            "workspace_error",
            "Set --workspace to a writable directory.",
        )
    })?;
    let root = home.join("Library/Application Support/app.picaso.cli");
    #[cfg(feature = "fixture-provider")]
    let root = root.join("fixture-workspace");
    Ok(root)
}

fn open_library(root: &Path) -> Result<Library> {
    Library::open(root).map_err(|e| {
        let code = if e.is::<WorkspaceBusy>() {
            "workspace_busy"
        } else {
            "workspace_error"
        };
        CliError::new(code, e)
    })
}

fn find_job(lib: &Library, id: Uuid) -> Result<Option<Job>> {
    lib.job(id).map_err(|e| CliError {
        job_id: Some(id),
        ..CliError::new("workspace_error", e)
    })
}

fn original(lib: &Library, id: &str) -> Result<ImageFile> {
    let (format, path) = lib
        .image_file(id)
        .map_err(|e| CliError::new("image_error", e))?;
    if !path.is_file() {
        return Err(CliError::new(
            "outputs_missing",
            format!("Original {id} is missing from the workspace."),
        ));
    }
    json_path(&path)?;
    Ok(ImageFile {
        image_id: id.to_owned(),
        format,
        path,
    })
}

fn job_result(job: Job, images: Vec<ImageFile>) -> Value {
    json!({"jobId": job.id, "status": job.status, "model": job.model, "provider": job.provider,
        "prompt": job.prompt, "inputId": job.input_id, "parameters": job.parameters, "images": images,
        "message": job.message, "usage": job.usage})
}

fn originals(lib: &Library, job: &Job) -> Result<Vec<ImageFile>> {
    job.images.iter().map(|id| original(lib, id)).collect()
}

// JSON paths must round-trip so the next command can open the same file.
fn json_path(path: &Path) -> Result<()> {
    if path.to_str().is_none() {
        return Err(CliError::new(
            "invalid_path",
            "JSON output requires UTF-8 paths.",
        ));
    }
    Ok(())
}

fn output_directory(lib: &Library, dir: &Path) -> anyhow::Result<PathBuf> {
    let dir = std::path::absolute(dir)?;
    // Check the existing ancestor before creating directories inside a managed library.
    if let Some(parent) = dir.ancestors().find(|p| p.exists()) {
        lib.export_directory(parent)?;
    }
    fs::create_dir_all(&dir)?;
    let dir = lib.export_directory(&dir)?;
    anyhow::ensure!(
        dir.to_str().is_some(),
        "JSON output requires a UTF-8 export path."
    );
    Ok(dir)
}

fn export_outputs(lib: &Library, dir: &Path, images: &mut [ImageFile]) -> anyhow::Result<()> {
    let dir = output_directory(lib, dir)?;
    for image in images {
        let dest = dir.join(format!("{}.{}", image.image_id, image.format.extension()));
        lib.export_new(&image.image_id, &dest)?;
        image.path = dest;
    }
    Ok(())
}

fn submit_new(
    lib: &Library,
    model: picaso_core::catalog::Model,
    draft: &Submission,
) -> Result<Job> {
    #[cfg(not(feature = "fixture-provider"))]
    let provider = {
        let key = std::env::var("OPENAI_API_KEY")
            .ok()
            .filter(|k| !k.trim().is_empty())
            .ok_or_else(|| {
                CliError::new("missing_key", "Set OPENAI_API_KEY in the calling process.")
            })?;
        picaso_core::providers::openai::OpenAi::new(key, model)
            .map_err(|e| CliError::new("provider_error", e))?
    };
    #[cfg(feature = "fixture-provider")]
    let provider = {
        let _ = model;
        picaso_core::providers::Fixture::default()
    };
    let id = draft.request_id;
    jobs::submit(lib, &provider, draft).map_err(|e| CliError::submission(e, id))?;
    find_job(lib, id)?.ok_or_else(|| CliError {
        job_id: Some(id),
        ..CliError::new("workspace_error", "Could not find the submitted job.")
    })
}

fn submit(root: &Path, args: SubmitArgs, input: Option<Uuid>) -> Result<Value> {
    // Wait for the caller's pipe before taking the workspace lock.
    let prompt = args
        .read_prompt()
        .map_err(|e| CliError::new("invalid_prompt", e))?;
    let lib = open_library(root)?;
    json_path(
        &root
            .canonicalize()
            .map_err(|e| CliError::new("workspace_error", e))?,
    )?;
    let model = Catalog(root.join("models.json"))
        .select(MODELS, args.model.as_deref())
        .map_err(|e| CliError::new("model_error", format!("{e:#}")))?;
    let id = args.request_id.unwrap_or_else(Uuid::new_v4);
    let draft = Submission {
        request_id: id,
        prompt,
        model: Some(model.id.clone()),
        options: args.options(model.defaults.clone()),
        input_id: input.map(|id| id.to_string()),
    };
    let job = if let Some(prior) = find_job(&lib, id)? {
        prior
            .check_replay(&draft, &model)
            .map_err(|e| CliError::submission(e, id))?;
        prior
    } else {
        if let Some(dir) = &args.out_dir {
            output_directory(&lib, dir).map_err(|e| CliError::new("invalid_destination", e))?;
        }
        submit_new(&lib, model, &draft)?
    };
    let mut images = originals(&lib, &job).map_err(|e| CliError {
        job_id: Some(id),
        ..e
    })?;
    if images.is_empty() {
        return Err(CliError {
            job_id: Some(id),
            ..CliError::new(
                "outputs_missing",
                "All outputs of this job have been deleted.",
            )
        });
    }
    if let Some(dir) = args.out_dir {
        if let Err(e) = export_outputs(&lib, &dir, &mut images) {
            return Err(CliError {
                job_id: Some(id),
                images,
                ..CliError::new("export_failed", e)
            });
        }
    }
    Ok(job_result(job, images))
}

fn run(cli: Cli) -> Result<Value> {
    let root = workspace(cli.workspace)?;
    match cli.command {
        Command::Models => {
            let models = Catalog(root.join("models.json"))
                .available(MODELS)
                .map_err(|e| CliError::new("model_error", format!("{e:#}")))?;
            Ok(json!({"models": models}))
        }
        Command::Generate(args) => submit(&root, args, None),
        Command::Edit { image_id, args } => submit(&root, args, Some(image_id)),
        Command::Show { job_id } => {
            let lib = open_library(&root)?;
            let job = find_job(&lib, job_id)?.ok_or_else(|| {
                CliError::new("job_not_found", "No job with that ID in this workspace.")
            })?;
            let images = originals(&lib, &job)?;
            Ok(job_result(job, images))
        }
        Command::Export { image_id, out } => {
            let lib = open_library(&root)?;
            let mut image = original(&lib, &image_id.to_string())?;
            let dest =
                std::path::absolute(out).map_err(|e| CliError::new("invalid_destination", e))?;
            json_path(&dest)?;
            lib.export_new(&image.image_id, &dest)
                .map_err(|e| CliError::new("export_failed", e))?;
            image.path = dest;
            Ok(json!({"status":"succeeded", "images":[image]}))
        }
    }
}

fn main() -> ExitCode {
    let (result, compact) = match Cli::try_parse() {
        Ok(cli) => {
            let compact = cli.json;
            (run(cli), compact)
        }
        Err(e) if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) => {
            e.exit();
        }
        Err(e) => (Err(CliError::new("invalid_arguments", e)), true),
    };
    let (output, code) = match result {
        Ok(value) => (value, ExitCode::SUCCESS),
        Err(e) => (json!({"status":"error", "error":e}), ExitCode::FAILURE),
    };
    let mut stdout = io::stdout().lock();
    let written = if compact {
        serde_json::to_writer(&mut stdout, &output)
    } else {
        serde_json::to_writer_pretty(&mut stdout, &output)
    };
    if written.is_err() || writeln!(stdout).is_err() || stdout.flush().is_err() {
        eprintln!("Could not write the JSON result to stdout.");
        return ExitCode::FAILURE;
    }
    code
}
