use crate::error::{BeatPartnerError, Result};
use crossbeam::channel::Sender;
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

/// Database operations are sent to a dedicated background thread
/// to avoid blocking the UI or audio thread.
#[derive(Debug, Clone)]
pub enum DbCommand {
    GetSettings { key: String },
    SetSettings { key: String, value: String },
    CreateProject { name: String },
    GetProject { id: i64 },
    ListProjects,
}

#[derive(Debug, Clone)]
pub enum DbResponse {
    SettingsValue(Option<String>),
    ProjectId(i64),
    ProjectList(Vec<(i64, String)>),
    Ok,
    Error(String),
}

/// Handle to the background database thread.
pub struct DbService {
    tx: Sender<(DbCommand, Sender<DbResponse>)>,
    _handle: JoinHandle<()>,
}

impl DbService {
    /// Start the background DB thread and run migrations.
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let (tx, rx): (Sender<(DbCommand, Sender<DbResponse>)>, _) =
            crossbeam::channel::unbounded();

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let handle = thread::spawn(move || {
            let conn = match open_connection(&db_path) {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to open database: {}", e);
                    return;
                }
            };

            if let Err(e) = run_migrations(&conn) {
                log::error!("Failed to run migrations: {}", e);
                return;
            }

            log::info!("Database initialized at {:?}", db_path);

            while let Ok((cmd, resp_tx)) = rx.recv() {
                let resp = handle_command(&conn, cmd);
                let _ = resp_tx.send(resp);
            }
        });

        Ok(Self {
            tx,
            _handle: handle,
        })
    }

    /// Send a command and block until a response is received.
    pub fn request(&self, cmd: DbCommand) -> Result<DbResponse> {
        let (resp_tx, resp_rx) = crossbeam::channel::bounded(1);
        self.tx
            .send((cmd, resp_tx))
            .map_err(|_| BeatPartnerError::Database(rusqlite::Error::InvalidQuery))?;
        resp_rx
            .recv()
            .map_err(|_| BeatPartnerError::Database(rusqlite::Error::InvalidQuery))
    }

    /// Send a command without waiting for a response (fire-and-forget).
    pub fn send(&self, cmd: DbCommand) -> Result<()> {
        let (resp_tx, _resp_rx) = crossbeam::channel::bounded(1);
        self.tx
            .send((cmd, resp_tx))
            .map_err(|_| BeatPartnerError::Database(rusqlite::Error::InvalidQuery))?;
        Ok(())
    }
}

fn open_connection(path: &PathBuf) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    let mut stmt = conn.prepare("SELECT name FROM _migrations ORDER BY id")?;
    let applied: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(BeatPartnerError::Database)?;

    let migrations_dir = std::path::Path::new("migrations");
    let mut migration_files: Vec<_> = std::fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension().map_or(false, |ext| ext == "sql")
        })
        .collect();

    migration_files.sort_by_key(|e| e.file_name());

    for entry in migration_files {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        if applied.contains(&name) {
            continue;
        }

        let sql = std::fs::read_to_string(&path)?;
        conn.execute_batch(&sql)?;
        conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [&name])?;

        log::info!("Applied migration: {}", name);
    }

    Ok(())
}

fn handle_command(conn: &Connection, cmd: DbCommand) -> DbResponse {
    match cmd {
        DbCommand::GetSettings { key } => {
            let result = conn
                .query_row("SELECT value FROM settings WHERE key = ?1", [&key], |row| {
                    row.get::<_, String>(0)
                })
                .ok();
            DbResponse::SettingsValue(result)
        }
        DbCommand::SetSettings { key, value } => {
            let res = conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP",
                [&key, &value],
            );
            match res {
                Ok(_) => DbResponse::Ok,
                Err(e) => DbResponse::Error(e.to_string()),
            }
        }
        DbCommand::CreateProject { name } => {
            let res = conn.execute("INSERT INTO projects (name) VALUES (?1)", [&name]);
            match res {
                Ok(_) => match conn.last_insert_rowid() {
                    id if id > 0 => DbResponse::ProjectId(id),
                    _ => DbResponse::Error("Failed to get last insert rowid".to_string()),
                },
                Err(e) => DbResponse::Error(e.to_string()),
            }
        }
        DbCommand::GetProject { id } => {
            let result = conn
                .query_row("SELECT id, name FROM projects WHERE id = ?1", [id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })
                .ok();
            match result {
                Some((id, name)) => DbResponse::ProjectList(vec![(id, name)]),
                None => DbResponse::ProjectList(vec![]),
            }
        }
        DbCommand::ListProjects => {
            let mut stmt =
                match conn.prepare("SELECT id, name FROM projects ORDER BY updated_at DESC") {
                    Ok(s) => s,
                    Err(e) => return DbResponse::Error(e.to_string()),
                };
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            });
            match rows {
                Ok(iter) => {
                    let projects: Vec<_> = iter.filter_map(|r| r.ok()).collect();
                    DbResponse::ProjectList(projects)
                }
                Err(e) => DbResponse::Error(e.to_string()),
            }
        }
    }
}
