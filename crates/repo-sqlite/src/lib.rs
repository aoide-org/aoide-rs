// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#![warn(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(rust_2018_idioms)]
#![deny(rust_2021_compatibility)]
#![deny(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::all)]
#![deny(clippy::explicit_deref_methods)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::must_use_candidate)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use diesel::{QueryResult, RunQueryDsl as _, SqliteConnection};

// The following workaround is need to avoid cluttering the code with
// #[cfg_attr(feature = "diesel", ...)] to specify custom diesel
// attributes.
#[macro_use]
extern crate diesel;

// Workaround for using the embed_migrations!() macro in tests.
#[cfg(test)]
#[macro_use]
extern crate diesel_migrations;

pub mod prelude {
    pub(crate) use crate::util::{clock::*, entity::*, *};
    pub(crate) use aoide_repo::prelude::*;
    pub(crate) use diesel::{prelude::*, result::Error as DieselError, SqliteConnection};
    pub(crate) use semval::prelude::*;
    pub(crate) use std::ops::Deref;

    pub use diesel::Connection as _;

    #[derive(Clone, Copy)]
    #[allow(missing_debug_implementations)]
    pub struct Connection<'db>(&'db SqliteConnection);

    impl<'db> Connection<'db> {
        pub const fn new(inner: &'db SqliteConnection) -> Self {
            Self(inner)
        }
    }

    impl<'db> From<&'db SqliteConnection> for Connection<'db> {
        fn from(inner: &'db SqliteConnection) -> Self {
            Self::new(inner)
        }
    }

    impl<'db> AsRef<SqliteConnection> for Connection<'db> {
        fn as_ref(&self) -> &SqliteConnection {
            self.0
        }
    }

    impl<'db> Deref for Connection<'db> {
        type Target = SqliteConnection;

        fn deref(&self) -> &Self::Target {
            self.as_ref()
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
        use super::SqliteConnection;

        use diesel::Connection as _;

        pub type TestResult<T> = anyhow::Result<T>;

        embed_migrations!("migrations");

        pub fn establish_connection() -> TestResult<SqliteConnection> {
            let connection =
                SqliteConnection::establish(":memory:").expect("in-memory database connection");
            embedded_migrations::run(&connection)?;
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
pub fn initialize_database(connection: &SqliteConnection) -> QueryResult<()> {
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
