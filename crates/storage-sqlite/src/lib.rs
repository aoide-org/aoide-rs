// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![deny(missing_debug_implementations)]
#![deny(clippy::clone_on_ref_ptr)]
#![warn(rust_2018_idioms)]

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use thiserror::Error;

#[cfg(feature = "with-tokio")]
pub mod tokio;

pub type SqliteConnectionManager = ConnectionManager<SqliteConnection>;
pub type SqliteConnectionPool = Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = PooledConnection<SqliteConnectionManager>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] diesel::result::Error),

    #[error(transparent)]
    Connection(#[from] r2d2::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[cfg(feature = "with-tokio")]
    #[error("timeout: {reason}")]
    Timeout { reason: String },

    #[cfg(feature = "with-tokio")]
    #[error(transparent)]
    TaskScheduling(#[from] ::tokio::task::JoinError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn initialize_database(connection: &SqliteConnection) -> Result<()> {
    log::info!("Initializing database");
    diesel::sql_query(r#"
PRAGMA journal_mode = WAL;        -- better write-concurrency
PRAGMA synchronous = NORMAL;      -- fsync only in critical moments, safe for journal_mode = WAL
PRAGMA wal_autocheckpoint = 1000; -- write WAL changes back every 1000 pages (default), for an in average 1MB WAL file
PRAGMA wal_checkpoint(TRUNCATE);  -- free some space by truncating possibly massive WAL files from the last run
PRAGMA secure_delete = 0;         -- avoid some disk I/O
PRAGMA automatic_index = 1;       -- detect and log missing indexes
PRAGMA foreign_keys = 1;          -- check foreign key constraints
PRAGMA defer_foreign_keys = 1;    -- delay enforcement of foreign key constraints until commit
PRAGMA recursive_triggers = 1;    -- for recursive ON CASCADE DELETE actions
PRAGMA encoding = 'UTF-8';
"#).execute(connection)?;
    Ok(())
}

pub fn vacuum_database(connection: &SqliteConnection) -> Result<()> {
    diesel::dsl::sql_query("VACUUM")
        .execute(connection)
        .map(|_count| {
            debug_assert_eq!(0, _count);
        })
        .map_err(Into::into)
}

/// Gather statistics about the schema and generate hints
/// for the query planner.
///
/// See also: https://www.sqlite.org/lang_analyze.html
/// "Statistics gathered by ANALYZE are not automatically updated
/// as the content of the database changes. If the content of the
/// database changes significantly, or if the database schema changes,
/// then one should consider rerunning the ANALYZE command in order
/// to update the statistics.
pub fn analyze_and_optimize_database_stats(connection: &SqliteConnection) -> Result<()> {
    diesel::dsl::sql_query("ANALYZE")
        .execute(connection)
        .map(|_| ())
        .map_err(Into::into)
}

pub fn cleanse_database(connection: &SqliteConnection, vacuum: bool) -> Result<()> {
    // According to Richard Hipp himself executing VACUUM before ANALYZE is the
    // recommended order: https://sqlite.org/forum/forumpost/62fb63a29c5f7810?t=h
    if vacuum {
        log::info!("Rebuilding database storage");
        vacuum_database(connection)?;
    }

    log::info!("Analyzing and optimizing database statistics");
    analyze_and_optimize_database_stats(connection)?;

    Ok(())
}

pub fn create_database_connection_pool(
    database_url: &str,
    max_size: u32,
) -> Result<SqliteConnectionPool> {
    log::info!("Creating SQLite connection pool");
    let manager = SqliteConnectionManager::new(database_url);
    let pool = SqliteConnectionPool::builder()
        .max_size(max_size)
        .build(manager)?;
    Ok(pool)
}

pub fn get_pooled_database_connection(
    pool: &SqliteConnectionPool,
) -> Result<SqlitePooledConnection> {
    pool.get().map_err(Into::into)
}
