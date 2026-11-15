// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::{
    RunQueryDsl as _, SqliteConnection,
    sqlite::pragma::{IncrementalVacuum, SchemaPragma as _},
};
use thiserror::Error;

pub mod connection;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] diesel::result::Error),

    #[error(transparent)]
    DatabaseConnection(#[from] diesel::ConnectionError),

    #[error(transparent)]
    DatabaseConnectionPool(#[from] r2d2::Error),

    #[error(transparent)]
    Other(anyhow::Error),

    #[cfg(feature = "tokio")]
    #[error("timeout: {reason}")]
    TaskTimeout { reason: String },

    #[cfg(feature = "tokio")]
    #[error(transparent)]
    TaskScheduling(#[from] tokio::task::JoinError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VacuumMode {
    Full,
    Incremental,
}

pub fn vacuum_database(connection: &mut SqliteConnection, mode: VacuumMode) -> Result<()> {
    match mode {
        VacuumMode::Full => connection.vacuum(None),
        // Releases the whole freelist. Unlike executing `PRAGMA incremental_vacuum`
        // as a prepared statement, which only frees a single page per step, this
        // runs the pragma to completion.
        VacuumMode::Incremental => IncrementalVacuum::set(None, None, connection),
    }
    .map_err(Into::into)
}

/// Gather statistics about the schema and generate hints
/// for the query planner.
///
/// See also: <https://www.sqlite.org/lang_analyze.html/>
/// "Statistics gathered by ANALYZE are not automatically updated
/// as the content of the database changes. If the content of the
/// database changes significantly, or if the database schema changes,
/// then one should consider rerunning the ANALYZE command in order
/// to update the statistics.
pub fn analyze_and_optimize_database_stats(connection: &mut SqliteConnection) -> Result<()> {
    diesel::dsl::sql_query("ANALYZE")
        .execute(connection)
        .map(|_| ())
        .map_err(Into::into)
}

pub fn cleanse_database(
    connection: &mut SqliteConnection,
    vacuum_mode: Option<VacuumMode>,
) -> Result<()> {
    // According to Richard Hipp himself executing VACUUM before ANALYZE is the
    // recommended order: https://sqlite.org/forum/forumpost/62fb63a29c5f7810?t=h
    if let Some(vacuum_mode) = vacuum_mode {
        log::info!("Rebuilding database storage before analysis & optimization");
        vacuum_database(connection, vacuum_mode)?;
    }

    log::info!("Analyzing and optimizing database statistics");
    analyze_and_optimize_database_stats(connection)?;

    Ok(())
}

#[cfg(test)]
mod tests;
