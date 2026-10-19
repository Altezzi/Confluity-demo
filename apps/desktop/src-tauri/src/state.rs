use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;

use crate::db::{open_db, WorkspacePaths, DbResult};

pub struct AppState {
    pub paths: WorkspacePaths,
    pub conn: Mutex<Connection>,
}

impl AppState {
    pub fn init() -> DbResult<Arc<Self>> {
        let paths = WorkspacePaths::resolve()?;
        let conn = open_db(&paths.db)?;
        crate::db::seed::ensure_seeded(&conn)?;
        Ok(Arc::new(Self {
            paths,
            conn: Mutex::new(conn),
        }))
    }
}
