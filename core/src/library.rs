use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use uuid::Uuid;

use crate::jobs::{Submission, SubmissionError};
use crate::media::{Format, Original};
use crate::providers::{Model, Output};
pub const UNCERTAIN: &str = "Result unconfirmed. Submitting again may incur another charge.";

#[derive(Debug, thiserror::Error)]
#[error("The workspace is already open in another Picaso process.")]
pub struct WorkspaceBusy;

#[derive(Clone)]
pub struct Library {
    root: PathBuf,
    // Hold this for the process lifetime so startup recovery cannot interrupt another writer.
    _lock: Arc<fs::File>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub prompt: String,
    pub provider: String,
    pub model: String,
    pub status: String,
    pub input_id: Option<String>,
    pub created: i64,
    pub message: Option<String>,
    pub images: Vec<String>,
    pub parameters: serde_json::Value,
    pub usage: Option<serde_json::Value>,
}
impl Job {
    /// A matching, completed request can be reused without contacting its provider.
    pub fn check_replay(&self, draft: &Submission, model: &Model) -> Result<()> {
        let mut stored = self.parameters.clone();
        // v1 tasks used the API's implicit auto background.
        if stored.get("background").is_none() {
            stored["background"] = serde_json::json!("auto");
        }
        if self.prompt != draft.prompt.trim()
            || self.model != model.id
            || self.provider != model.provider
            || self.input_id != draft.input_id
            || stored != serde_json::to_value(&draft.options)?
        {
            return Err(SubmissionError::Conflict.into());
        }
        match self.status.as_str() {
            "succeeded" => Ok(()),
            "unconfirmed" => Err(SubmissionError::Unconfirmed(
                self.message.clone().unwrap_or_else(|| UNCERTAIN.into()),
            )
            .into()),
            "failed" => Err(SubmissionError::Failed(
                self.message
                    .clone()
                    .unwrap_or_else(|| "This request failed.".into()),
            )
            .into()),
            "running" => Err(SubmissionError::Running.into()),
            _ => bail!("Unknown job status."),
        }
    }
}

impl Library {
    pub fn open(root: &Path) -> Result<Self> {
        fs::create_dir_all(root.join("originals"))?;
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join("library.lock"))?;
        match lock.try_lock() {
            Ok(()) => (),
            Err(fs::TryLockError::WouldBlock) => return Err(WorkspaceBusy.into()),
            Err(fs::TryLockError::Error(e)) => return Err(e.into()),
        }
        let lib = Self {
            root: root.canonicalize()?,
            _lock: Arc::new(lock),
        };
        let mut conn = lib.connect()?;
        let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        // Rebuild images with a nullable producing job. Existing jobs reference
        // this table, so check their foreign keys before committing the rebuild.
        if (1..=3).contains(&version) {
            conn.pragma_update(None, "foreign_keys", false)?;
        }
        let tx = conn.transaction()?;
        match version {
            0 => tx.execute_batch("CREATE TABLE jobs (
                id TEXT PRIMARY KEY, prompt TEXT NOT NULL, provider TEXT NOT NULL, model TEXT NOT NULL,
                status TEXT NOT NULL, parameters TEXT NOT NULL, input_id TEXT REFERENCES images(id),
                created INTEGER NOT NULL DEFAULT(unixepoch()), message TEXT, usage TEXT);
                CREATE TABLE images (id TEXT PRIMARY KEY, job TEXT REFERENCES jobs(id), format TEXT NOT NULL DEFAULT 'png', deleted INTEGER NOT NULL DEFAULT 0);
                CREATE INDEX images_job ON images(job);")?,
            1 => tx.execute_batch("ALTER TABLE images ADD COLUMN format TEXT NOT NULL DEFAULT 'png';")?,
            2..=4 => (),
            _ => bail!("This library needs a newer version of Picaso."),
        }
        if (1..=3).contains(&version) {
            if version < 3 {
                tx.execute_batch(
                    "ALTER TABLE images ADD COLUMN deleted INTEGER NOT NULL DEFAULT 0;",
                )?;
            }
            tx.execute_batch("CREATE TABLE new_images (id TEXT PRIMARY KEY, job TEXT REFERENCES jobs(id), format TEXT NOT NULL DEFAULT 'png', deleted INTEGER NOT NULL DEFAULT 0);
                INSERT INTO new_images(rowid,id,job,format,deleted) SELECT rowid,id,job,format,deleted FROM images;
                DROP TABLE images;
                ALTER TABLE new_images RENAME TO images;
                CREATE INDEX images_job ON images(job);")?;
            if tx.prepare("PRAGMA foreign_key_check")?.exists([])? {
                bail!("The library contains a missing image or task reference.");
            }
        }
        if version < 4 {
            tx.execute_batch("CREATE INDEX jobs_input ON jobs(input_id); PRAGMA user_version=4;")?;
        }
        // Drafts live in the webview. Recover abandoned imports after an exit.
        tx.execute("DELETE FROM images WHERE job IS NULL AND NOT EXISTS (SELECT 1 FROM jobs WHERE input_id=images.id)", [])?;
        tx.execute(
            "UPDATE jobs SET status='unconfirmed', message=?1 WHERE status='running'",
            [UNCERTAIN],
        )?;
        tx.commit()?;
        for item in fs::read_dir(root.join("originals"))? {
            let path = item?.path();
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let orphan = if let Some(id) = name
                .rsplit_once('.')
                .and_then(|(stem, ext)| Format::parse(ext).and_then(|_| Uuid::parse_str(stem).ok()))
            {
                !lib.has_image(&id.to_string())?
            } else {
                name.starts_with(".picaso-tmp-")
            };
            if orphan && path.is_file() {
                fs::remove_file(path)?;
            }
        }
        Ok(lib)
    }
    pub(crate) fn connect(&self) -> Result<Connection> {
        let conn = Connection::open(self.root.join("library.sqlite3"))?;
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        Ok(conn)
    }
    pub(crate) fn reserve(&self, draft: &Submission, model: &Model) -> Result<bool> {
        let added = self.connect()?.execute(
            "INSERT INTO jobs(id,prompt,provider,model,status,parameters,input_id)
            VALUES (?1,?2,?3,?4,'running',?5,?6) ON CONFLICT(id) DO NOTHING",
            params![
                draft.request_id.to_string(),
                draft.prompt.trim(),
                model.provider,
                model.id,
                serde_json::to_string(&draft.options)?,
                draft.input_id
            ],
        )?;
        if added == 0 {
            self.job(draft.request_id)?
                .context("The existing job is missing.")?
                .check_replay(draft, model)?;
        }
        Ok(added != 0)
    }
    pub(crate) fn finish(&self, id: &Uuid, output: Output, format: Format) -> Result<()> {
        if output.images.is_empty() {
            bail!("No image was returned.");
        }
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        let mut written = Vec::new();
        let result = (|| -> Result<()> {
            for data in output.images {
                format.validate(&data)?;
                let (image_id, path) = self.write_original(&data, format)?;
                written.push(path);
                tx.execute(
                    "INSERT INTO images(id,job,format) VALUES (?1,?2,?3)",
                    params![image_id, id.to_string(), format.extension()],
                )?;
            }
            tx.execute(
                "UPDATE jobs SET status='succeeded',usage=?1 WHERE id=?2",
                params![output.usage.map(|v| v.to_string()), id.to_string()],
            )?;
            tx.commit()?;
            Ok(())
        })();
        if result.is_err() {
            for path in written {
                let _ = fs::remove_file(path);
            }
        }
        result
    }
    fn write_original(&self, bytes: &[u8], format: Format) -> Result<(String, PathBuf)> {
        let id = Uuid::new_v4().to_string();
        let dir = self.root.join("originals");
        let path = dir.join(format!("{id}.{}", format.extension()));
        let mut tmp = tempfile::Builder::new()
            .prefix(".picaso-tmp-")
            .tempfile_in(dir)?;
        tmp.write_all(bytes)?;
        tmp.as_file().sync_all()?;
        tmp.persist_noclobber(&path)?;
        Ok((id, path))
    }
    pub fn import_reference(&self, bytes: &[u8]) -> Result<String> {
        let format = Format::reference(bytes)?;
        let conn = self.connect()?;
        let (id, path) = self.write_original(bytes, format)?;
        if let Err(err) = conn.execute(
            "INSERT INTO images(id,format) VALUES (?1,?2)",
            params![id, format.extension()],
        ) {
            let _ = fs::remove_file(path);
            return Err(err.into());
        }
        Ok(id)
    }
    pub fn discard_reference(&self, id: Uuid) -> Result<()> {
        let ext: Option<String> = self.connect()?.query_row(
            "DELETE FROM images WHERE id=?1 AND job IS NULL AND NOT EXISTS (SELECT 1 FROM jobs WHERE input_id=?1) RETURNING format",
            [id.to_string()], |r| r.get(0)).optional()?;
        if let Some(ext) = ext {
            self.remove_original(id, &ext)?;
        }
        Ok(())
    }
    pub(crate) fn fail(&self, id: &Uuid, message: &str, uncertain: bool) -> Result<()> {
        self.connect()?.execute(
            "UPDATE jobs SET status=?1,message=?2 WHERE id=?3",
            params![
                if uncertain { "unconfirmed" } else { "failed" },
                message,
                id.to_string()
            ],
        )?;
        Ok(())
    }
    fn has_image(&self, id: &str) -> Result<bool> {
        Ok(self
            .connect()?
            .query_row(
                "SELECT 1 FROM images WHERE id=?1 AND deleted=0",
                [id],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }
    pub fn image_format(&self, id: &str) -> Result<Format> {
        let ext: String = self
            .connect()?
            .query_row(
                "SELECT format FROM images WHERE id=?1 AND deleted=0",
                [id],
                |r| r.get(0),
            )
            .optional()?
            .context("This image is missing from the library.")?;
        Format::parse(&ext).context("Unknown library image format.")
    }
    pub fn image_file(&self, id: &str) -> Result<(Format, PathBuf)> {
        let id = Uuid::parse_str(id)
            .context("Choose an image from History.")?
            .to_string();
        let format = self.image_format(&id)?;
        let path = self
            .root
            .join("originals")
            .join(format!("{id}.{}", format.extension()));
        Ok((format, path))
    }
    pub fn original(&self, id: &str) -> Result<Original> {
        let (format, path) = self.image_file(id)?;
        let bytes = fs::read(path).context(
            "The original image is missing or unreadable. Restore it or choose another image.",
        )?;
        Ok(Original { bytes, format })
    }
    pub fn export_directory(&self, dir: &Path) -> Result<PathBuf> {
        let dir = dir.canonicalize()?;
        anyhow::ensure!(dir.is_dir(), "Choose a destination folder.");
        if dir.starts_with(&self.root) {
            bail!("Choose a location outside Picaso’s data folder.");
        }
        Ok(dir)
    }
    pub fn export(&self, id: &str, dest: &Path) -> Result<()> {
        self.write_export(id, dest, true)
    }
    /// Automation can replay the same export while preserving unrelated files.
    pub fn export_new(&self, id: &str, dest: &Path) -> Result<()> {
        self.write_export(id, dest, false)
    }
    fn write_export(&self, id: &str, dest: &Path, replace: bool) -> Result<()> {
        let dir = self.export_directory(dest.parent().context("Choose a destination folder.")?)?;
        let data = self.original(id)?;
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        tmp.write_all(&data.bytes)?;
        if replace {
            tmp.persist(dest)?;
        } else if let Err(error) = tmp.persist_noclobber(dest) {
            if error.error.kind() == std::io::ErrorKind::AlreadyExists {
                let metadata = dest.symlink_metadata()?;
                if metadata.is_file()
                    && metadata.len() == data.bytes.len() as u64
                    && fs::read(dest)? == data.bytes
                {
                    return Ok(());
                }
                bail!("The export destination already exists with different content. Choose another path.");
            }
            return Err(error.into());
        }
        Ok(())
    }
    pub fn delete_image(&self, id: Uuid) -> Result<()> {
        // Keep the ID for edit lineage and submission deduplication. Commit the
        // deletion first so startup recovery can finish an interrupted unlink.
        let ext: String = self
            .connect()?
            .query_row(
                "UPDATE images SET deleted=1 WHERE id=?1 RETURNING format",
                [id.to_string()],
                |r| r.get(0),
            )
            .optional()?
            .context("This image is missing from the library.")?;
        self.remove_original(id, &ext)
    }
    fn remove_original(&self, id: Uuid, ext: &str) -> Result<()> {
        let format = Format::parse(ext).context("Unknown library image format.")?;
        let path = self
            .root
            .join("originals")
            .join(format!("{id}.{}", format.extension()));
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e).context("Removed from the library, but the file could not be erased. Check the data folder permissions and try again."),
        }
    }
    pub fn history(&self, recent_only: bool) -> Result<Vec<Job>> {
        // Each job contributes at least one image or status tile to the recent gallery.
        let limit = if recent_only { "LIMIT 8" } else { "" };
        self.query_jobs(&format!("WHERE status != 'succeeded' OR EXISTS (SELECT 1 FROM images WHERE job=jobs.id AND deleted=0)
            ORDER BY created DESC, rowid DESC {limit}"), [])
    }
    pub fn job(&self, id: Uuid) -> Result<Option<Job>> {
        Ok(self.query_jobs("WHERE id=?1", [id.to_string()])?.pop())
    }
    fn query_jobs(&self, filter: &str, params: impl rusqlite::Params) -> Result<Vec<Job>> {
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        let mut stmt = tx.prepare(&format!("SELECT id,prompt,provider,model,status,input_id,created,message,parameters,usage FROM jobs {filter}"))?;
        let mut outputs =
            tx.prepare("SELECT id FROM images WHERE job=?1 AND deleted=0 ORDER BY rowid")?;
        let mut rows = stmt.query(params)?;
        let mut jobs = Vec::new();
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            jobs.push(Job {
                images: outputs
                    .query_map([&id], |r| r.get(0))?
                    .collect::<rusqlite::Result<_>>()?,
                id,
                prompt: row.get(1)?,
                provider: row.get(2)?,
                model: row.get(3)?,
                status: row.get(4)?,
                input_id: row.get(5)?,
                created: row.get(6)?,
                message: row.get(7)?,
                parameters: serde_json::from_str(&row.get::<_, String>(8)?)?,
                usage: row
                    .get::<_, Option<String>>(9)?
                    .map(|json| serde_json::from_str(&json))
                    .transpose()?,
            });
        }
        Ok(jobs)
    }
}
