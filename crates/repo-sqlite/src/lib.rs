// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::explicit_deref_methods)]
#![warn(clippy::explicit_into_iter_loop)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::must_use_candidate)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]
// recursion_limit was required for diesel
#![recursion_limit = "256"]

use diesel::{
    migration::{MigrationVersion, Result as MigrationResult},
    QueryResult, RunQueryDsl as _,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness as _};

pub type DbBackend = diesel::sqlite::Sqlite;
pub type DbConnection = diesel::sqlite::SqliteConnection;

pub mod prelude {
    pub(crate) use crate::util::{clock::*, entity::*, *};
    pub(crate) use aoide_repo::prelude::*;
    pub(crate) use diesel::{prelude::*, result::Error as DieselError};
    pub(crate) use semval::prelude::*;
    pub(crate) use std::ops::Deref;
    use std::ops::DerefMut;

    pub use crate::{DbBackend, DbConnection};

    pub use diesel::Connection as _;
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
        use DieselError::*;
        match err {
            NotFound => RepoError::NotFound,
            err => anyhow::Error::from(err).into(),
        }
    }

    #[derive(Debug)]
    pub struct DieselTransactionError<E>(E);

    impl<E> DieselTransactionError<E> {
        pub const fn new(inner: E) -> Self {
            Self(inner)
        }

        pub fn into_inner(self) -> E {
            let Self(inner) = self;
            inner
        }
    }

    impl<E> From<DieselError> for DieselTransactionError<E>
    where
        E: From<RepoError>,
    {
        fn from(err: DieselError) -> Self {
            Self(repo_error(err).into())
        }
    }

    impl<E> From<RepoError> for DieselTransactionError<E>
    where
        E: From<RepoError>,
    {
        fn from(err: RepoError) -> Self {
            Self(err.into())
        }
    }

    pub type RepoTransactionError = DieselTransactionError<RepoError>;

    pub(crate) use aoide_repo::RecordId as RowId;

    #[cfg(test)]
    pub mod tests {
        use super::DbConnection;
        use diesel::Connection as _;

        pub type TestResult<T> = anyhow::Result<T>;

        pub fn establish_connection() -> TestResult<DbConnection> {
            let mut connection =
                DbConnection::establish(":memory:").expect("in-memory database connection");
            crate::run_migrations(&mut connection)
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
            Ok(connection)
        }
    }
}

pub mod repo;

mod db;
mod util;

use prelude::Connection;

/// Configure the database engine
///
/// The implementation of the repositories and use cases relies on a proper
/// configuration of the database engine like the behavior, e.g. recursive
/// cascading deletes.
///
/// Some values like the text encoding can only be changed once after the
/// database has initially been created.
pub fn initialize_database(connection: &mut DbConnection) -> QueryResult<()> {
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

const EMBEDDED_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(connection: &mut DbConnection) -> MigrationResult<Vec<MigrationVersion<'_>>> {
    connection.run_pending_migrations(EMBEDDED_MIGRATIONS)
}
