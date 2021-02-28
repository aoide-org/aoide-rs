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
#![warn(rust_2018_idioms)]

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
            &self.0
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
