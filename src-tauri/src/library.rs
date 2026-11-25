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

use crate::media::{Format, Original};
use crate::providers::{Model, Output};
pub const UNCERTAIN: &str = "Result unconfirmed. Submitting again may incur another charge.";

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
impl Library {
    pub fn open(root: &Path) -> Result<Self> {
        fs::create_dir_all(root.join("originals"))?;
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join("library.lock"))?;
        lock.try_lock()
            .context("The library is already open in another Picaso window.")?;
        let lib = Self {
            root: root.into(),
            _lock: Arc::new(lock),
        };
        let mut conn = lib.connect()?;
        let tx = conn.transaction()?;
        match tx.pragma_query_value(None, "user_version", |r| r.get::<_,i64>(0))? {
            0 => tx.execute_batch("CREATE TABLE jobs (
                id TEXT PRIMARY KEY, prompt TEXT NOT NULL, provider TEXT NOT NULL, model TEXT NOT NULL,
                status TEXT NOT NULL, parameters TEXT NOT NULL, input_id TEXT REFERENCES images(id),
                created INTEGER NOT NULL DEFAULT(unixepoch()), message TEXT, usage TEXT);
                CREATE TABLE images (id TEXT PRIMARY KEY, job TEXT NOT NULL REFERENCES jobs(id), format TEXT NOT NULL DEFAULT 'png');
                CREATE INDEX images_job ON images(job);
                PRAGMA user_version=2;")?,
            1 => tx.execute_batch("ALTER TABLE images ADD COLUMN format TEXT NOT NULL DEFAULT 'png'; PRAGMA user_version=2;")?,
            2 => (),
            _ => bail!("This library needs a newer version of Picaso."),
        }
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
    pub(crate) fn reserve(
        &self,
        id: &Uuid,
        prompt: &str,
        model: Model,
        input: Option<&str>,
        parameters: &str,
    ) -> Result<bool> {
        let conn = self.connect()?;
        let added = conn.execute(
            "INSERT INTO jobs(id,prompt,provider,model,status,parameters,input_id)
            VALUES (?1,?2,?3,?4,'running',?5,?6) ON CONFLICT(id) DO NOTHING",
            params![
                id.to_string(),
                prompt,
                model.provider,
                model.id,
                parameters,
                input
            ],
        )?;
        if added == 0 {
            let (matches, status, message, stored): (bool,String,Option<String>,String) = conn.query_row(
                "SELECT prompt=?2 AND provider=?3 AND model=?4 AND input_id IS ?5,status,message,parameters FROM jobs WHERE id=?1",
                params![id.to_string(),prompt,model.provider,model.id,input], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
            let mut prior: serde_json::Value = serde_json::from_str(&stored)?;
            // v1 tasks used the API's implicit auto background.
            if prior.get("background").is_none() {
                prior["background"] = serde_json::json!("auto");
            }
            if !matches || prior != serde_json::from_str::<serde_json::Value>(parameters)? {
                bail!("This request ID was already used for a different submission.");
            }
            if status != "succeeded" {
                bail!(
                    "{}",
                    message.unwrap_or_else(|| "This request is already running.".into())
                );
            }
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
                let image_id = Uuid::new_v4().to_string();
                let path = self
                    .root
                    .join("originals")
                    .join(format!("{image_id}.{}", format.extension()));
                let mut tmp = tempfile::Builder::new()
                    .prefix(".picaso-tmp-")
                    .tempfile_in(self.root.join("originals"))?;
                tmp.write_all(&data)?;
                tmp.as_file().sync_all()?;
                tmp.persist_noclobber(&path)?;
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
            .query_row("SELECT 1 FROM images WHERE id=?1", [id], |_| Ok(()))
            .optional()?
            .is_some())
    }
    pub fn image_format(&self, id: &str) -> Result<Format> {
        let ext: String = self
            .connect()?
            .query_row("SELECT format FROM images WHERE id=?1", [id], |r| r.get(0))
            .optional()?
            .context("This image is missing from the library.")?;
        Format::parse(&ext).context("Unknown library image format.")
    }
    pub fn original(&self, id: &str) -> Result<Original> {
        let id = Uuid::parse_str(id)
            .context("Choose an image from History.")?
            .to_string();
        let format = self.image_format(&id)?;
        let bytes = fs::read(
            self.root
                .join("originals")
                .join(format!("{id}.{}", format.extension())),
        )
        .context(
            "The original image is missing or unreadable. Restore it or choose another image.",
        )?;
        Ok(Original { bytes, format })
    }
    pub fn export(&self, id: &str, dest: &Path) -> Result<()> {
        let data = self.original(id)?;
        if dest
            .parent()
            .and_then(|p| p.canonicalize().ok())
            .is_some_and(|p| p.starts_with(&self.root))
        {
            bail!("Choose a location outside Picaso’s data folder.");
        }
        let mut tmp = tempfile::NamedTempFile::new_in(
            dest.parent().context("Choose a destination folder.")?,
        )?;
        tmp.write_all(&data.bytes)?;
        tmp.persist(dest)?;
        Ok(())
    }
    pub fn history(&self, recent_only: bool) -> Result<Vec<Job>> {
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        // Eight successful jobs contain at least the eight most recent images when available.
        let filter = if recent_only {
            "WHERE status='succeeded' ORDER BY created DESC,rowid DESC LIMIT 8"
        } else {
            "ORDER BY created DESC,rowid DESC"
        };
        let mut stmt = tx.prepare(&format!("SELECT id,prompt,provider,model,status,input_id,created,message,parameters,usage FROM jobs {filter}"))?;
        let mut outputs = tx.prepare("SELECT id FROM images WHERE job=?1 ORDER BY rowid")?;
        let mut rows = stmt.query([])?;
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
