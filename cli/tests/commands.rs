use picaso_core::library::Library;
use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use tempfile::tempdir;
use uuid::Uuid;

// Each command runs from a fresh process, without a key or access to the user's workspace.
fn invoke(folder: &Path, args: &[&str], stdin: &str, success: bool) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_picaso"))
        .args(args)
        .arg("--json")
        .current_dir(folder)
        .env("HOME", folder)
        .env_remove("OPENAI_API_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let json: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|_| panic!("Expected JSON, got {:?}", output));
    assert_eq!(output.status.success(), success, "{json}");
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
    json
}

#[test]
fn commands_report_invalid_input_and_workspace_contention() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let id = Uuid::new_v4().to_string();
    let lib = Library::open(&root.join("shared")).unwrap();
    let busy = invoke(root, &["--workspace", "shared", "show", &id], "", false);
    assert_eq!(busy["error"]["code"], "workspace_busy");
    drop(lib);
    let missing = invoke(root, &["show", &id, "--workspace", "shared"], "", false);
    assert_eq!(missing["error"]["code"], "job_not_found");
    assert_eq!(
        invoke(root, &["generate"], "  \n", false)["error"]["code"],
        "invalid_prompt"
    );
    assert_eq!(
        invoke(root, &["edit", "--image-id", "invalid"], "", false)["error"]["code"],
        "invalid_arguments"
    );
    let inside_workspace = invoke(
        root,
        &[
            "--workspace",
            "shared",
            "generate",
            "--out-dir",
            "shared/models.json",
        ],
        "icon",
        false,
    );
    assert_eq!(inside_workspace["error"]["code"], "invalid_destination");
    assert!(!root.join("shared/models.json").exists());
}

#[cfg(not(feature = "fixture-provider"))]
#[test]
fn production_reads_workspace_models_and_requires_only_the_process_key() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let bundled = invoke(root, &["models"], "", true);
    let model_id = &bundled["models"][0]["id"];
    let config = serde_json::json!({"models":[{"id":model_id,"defaults":{"quality":"high"}}]});
    fs::write(root.join("models.json"), config.to_string()).unwrap();
    let configured = invoke(root, &["models", "--workspace", "."], "", true);
    assert_eq!(configured["models"][0]["defaults"]["quality"], "high");
    let failure = invoke(
        root,
        &["generate", "--key-source", "env"],
        "a portrait",
        false,
    );
    assert_eq!(failure["error"]["code"], "missing_key");
    assert!(root
        .join("Library/Application Support/app.picaso.cli/library.sqlite3")
        .is_file());
    assert!(!root
        .join("Library/Application Support/app.picaso.desktop")
        .exists());
}

#[cfg(feature = "fixture-provider")]
#[test]
fn stdin_generate_edit_second_original_export_and_replay() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    // The default CLI workspace is available even while the desktop library is locked.
    let desktop = root.join("Library/Application Support/app.picaso.desktop");
    let _desktop = Library::open(&desktop).unwrap();
    let id = Uuid::new_v4().to_string();
    let args = [
        "generate",
        "--prompt-file",
        "-",
        "--count",
        "2",
        "--request-id",
        &id,
        "--out-dir",
        "images",
        "--key-source",
        "env",
    ];
    let prompt = "一座白色的灯塔\n海面上有雾\n";
    let result = invoke(root, &args, prompt, true);
    assert_eq!(result["jobId"], id);
    assert_eq!(result["status"], "succeeded");
    let images = result["images"].as_array().unwrap();
    assert_eq!(images.len(), 2);
    let bytes = |image: &Value| {
        let path = Path::new(image["path"].as_str().unwrap());
        assert!(path.is_absolute());
        assert_eq!(
            path.parent().unwrap(),
            root.join("images").canonicalize().unwrap()
        );
        fs::read(path).unwrap()
    };
    let second = bytes(&images[1]);
    assert_ne!(bytes(&images[0]), second);
    assert_eq!(invoke(root, &args, prompt, true), result);
    assert_eq!(
        invoke(root, &args, "Another prompt", false)["error"]["code"],
        "request_conflict"
    );
    let image_id = images[1]["imageId"].as_str().unwrap();

    let edit = invoke(root, &["edit", "--image-id", image_id], "移走雾气", true);
    assert_eq!(edit["inputId"], image_id);
    assert_ne!(edit["jobId"], result["jobId"]);
    let edited = &edit["images"][0];
    assert_ne!(edited["imageId"], image_id);
    assert_eq!(edited["format"], "png");
    // The fixture echoes exactly the reference used by the edit.
    assert_eq!(fs::read(edited["path"].as_str().unwrap()).unwrap(), second);
    let saved = invoke(root, &["show", edit["jobId"].as_str().unwrap()], "", true);
    assert_eq!(saved, edit);

    let export = [
        "export",
        "--image-id",
        edited["imageId"].as_str().unwrap(),
        "--out",
        "final.png",
    ];
    let copy = invoke(root, &export, "", true);
    assert_eq!(copy["images"][0]["imageId"], edited["imageId"]);
    assert_eq!(fs::read(root.join("final.png")).unwrap(), second);
    fs::write(root.join("final.png"), "existing file").unwrap();
    assert_eq!(
        invoke(root, &export, "", false)["error"]["code"],
        "export_failed"
    );
    assert_eq!(fs::read(root.join("final.png")).unwrap(), b"existing file");

    let lib =
        Library::open(&root.join("Library/Application Support/app.picaso.cli/fixture-workspace"))
            .unwrap();
    let jobs = lib.history(false).unwrap();
    assert_eq!(jobs.len(), 2);
    assert_eq!(
        lib.job(Uuid::parse_str(&id).unwrap())
            .unwrap()
            .unwrap()
            .prompt,
        prompt.trim()
    );
}

#[cfg(feature = "fixture-provider")]
#[test]
fn interrupted_and_post_generation_export_failures_are_recoverable_without_resubmission() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let id = Uuid::new_v4().to_string();
    let submit = [
        "--workspace",
        "work",
        "generate",
        "--request-id",
        &id,
        "--out-dir",
        "exports",
    ];
    let result = invoke(root, &submit, "an icon", true);
    let image = &result["images"][0];
    fs::write(image["path"].as_str().unwrap(), "keep this file").unwrap();
    let error = invoke(root, &submit, "an icon", false);
    assert_eq!(error["error"]["code"], "export_failed");
    assert_eq!(error["error"]["jobId"], id);
    assert_eq!(error["error"]["images"][0]["imageId"], image["imageId"]);
    let copy = invoke(
        root,
        &[
            "--workspace",
            "work",
            "export",
            "--image-id",
            image["imageId"].as_str().unwrap(),
            "--out",
            "recovered.png",
        ],
        "",
        true,
    );
    assert_eq!(
        fs::read(copy["images"][0]["path"].as_str().unwrap()).unwrap(),
        fs::read(error["error"]["images"][0]["path"].as_str().unwrap()).unwrap()
    );

    // Seed a reserved job as it would appear if the CLI were killed after dispatch.
    let interrupted = Uuid::new_v4().to_string();
    let conn = rusqlite::Connection::open(root.join("work/library.sqlite3")).unwrap();
    conn.execute(
        "INSERT INTO jobs(id,prompt,provider,model,parameters,status)
        SELECT ?1,prompt,provider,model,parameters,'running' FROM jobs WHERE id=?2",
        [&interrupted, &id],
    )
    .unwrap();
    drop(conn);
    let replay = invoke(
        root,
        &[
            "generate",
            "--workspace",
            "work",
            "--request-id",
            &interrupted,
        ],
        "an icon",
        false,
    );
    assert_eq!(replay["error"]["code"], "unconfirmed");
    assert_eq!(replay["error"]["jobId"], interrupted);
    let job = invoke(
        root,
        &["show", &interrupted, "--workspace", "work"],
        "",
        true,
    );
    assert_eq!(job["status"], "unconfirmed");
    assert_eq!(job["images"].as_array().unwrap().len(), 0);
}
