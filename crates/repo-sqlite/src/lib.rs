// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// Suppress warnings for diesel AsChangeset
#![allow(clippy::ref_option_ref)]
// TODO: Remove temporary workaround.
// <https://github.com/rust-lang/rust-clippy/issues/11237>
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::wildcard_imports)]

use std::ops::{Deref, DerefMut};

use diesel::{
    migration::{MigrationVersion, Result as MigrationResult},
    prelude::*,
    result::Error as DieselError,
    QueryResult,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness as _};
use unicase::UniCase;

use aoide_repo::RepoError;
use aoide_storage_sqlite::VacuumMode;

mod db;

pub mod repo;

mod util;

pub(crate) use aoide_repo::RecordId as RowId;

const INIT_DB_SQL: &str = include_str!("init_db.sql");

pub(crate) const UNICASE_COLLATION_NAME: &str = "UNICASE";

pub const DEFAULT_VACUUM_MODE: VacuumMode = VacuumMode::Incremental;

pub type DbBackend = diesel::sqlite::Sqlite;
pub type DbConnection = diesel::sqlite::SqliteConnection;

#[allow(missing_debug_implementations)]
pub struct Connection<'db>(&'db mut DbConnection);

impl<'db> Connection<'db> {
    pub fn new(inner: &'db mut DbConnection) -> Self {
        Self(inner)
    }
}

impl<'db> From<&'db mut DbConnection> for Connection<'db> {
    fn from(inner: &'db mut DbConnection) -> Self {
        Self::new(inner)
    }
}

impl<'db> AsRef<DbConnection> for Connection<'db> {
    fn as_ref(&self) -> &DbConnection {
        self.0
    }
}

impl<'db> AsMut<DbConnection> for Connection<'db> {
    fn as_mut(&mut self) -> &mut DbConnection {
        self.0
    }
}

impl<'db> Deref for Connection<'db> {
    type Target = DbConnection;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'db> DerefMut for Connection<'db> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

pub(crate) fn repo_error(err: DieselError) -> RepoError {
    match err {
        DieselError::NotFound => RepoError::NotFound,
        err => RepoError::Other(err.into()),
    }
}

/// Configure the database engine
///
/// The implementation of the repositories and use cases relies on a proper
/// configuration of the database engine like the behavior, e.g. recursive
/// cascading deletes.
///
/// Some values like the text encoding can only be changed once after the
/// database has initially been created.
pub fn initialize_database(connection: &mut DbConnection) -> QueryResult<()> {
    diesel::sql_query(INIT_DB_SQL).execute(connection)?;

    // FIXME: How to use this collation for all LIKE queries instead of
    // the default NOCASE comparison?
    //
    // The built-in LIKE operator doesn't support case-insensitive matching with
    // custom collations beyond ASCII. You need to overload it separately using
    // sqlite3_create_function(). Both the 2-arg and 3-arg (escaped) versions of
    // LIKE are affected and need to be overloaded separately!
    //
    // Currently, "Beyonce" doesn't match "Beyoncé" and "Ä" doesn't match "ä".
    connection.register_collation(UNICASE_COLLATION_NAME, |lhs, rhs| {
        UniCase::new(lhs).cmp(&UniCase::new(rhs))
    })?;

    Ok(())
}

const EMBEDDED_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(connection: &mut DbConnection) -> MigrationResult<Vec<MigrationVersion<'_>>> {
    connection.run_pending_migrations(EMBEDDED_MIGRATIONS)
}

#[cfg(test)]
pub mod tests {
    use anyhow::anyhow;
    use diesel::Connection as _;

    use super::{initialize_database, repo_error, run_migrations, DbConnection};

    pub type TestResult<T> = anyhow::Result<T>;

    #[allow(clippy::missing_panics_doc)] // Never panics
    pub fn establish_connection() -> TestResult<DbConnection> {
        let mut connection =
            DbConnection::establish(":memory:").expect("in-memory database connection");
        initialize_database(&mut connection).map_err(repo_error)?;
        run_migrations(&mut connection).map_err(|err| anyhow!(err))?;
        Ok(connection)
    }
}
