// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::{RunQueryDsl as _, SqliteConnection};
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
    let sql = match mode {
        VacuumMode::Full => "VACUUM",
        VacuumMode::Incremental => "PRAGMA incremental_vacuum",
    };
    diesel::dsl::sql_query(sql)
        .execute(connection)
        .map(|count| {
            debug_assert_eq!(0, count);
        })
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
