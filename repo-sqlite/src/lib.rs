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

use anyhow::anyhow;
use diesel::{prelude::*, SqliteConnection};
use std::ops::Deref;

pub mod collection;
pub mod playlist;
pub mod track;
pub mod util;

#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct Connection<'db>(pub &'db SqliteConnection);

impl<'db> Connection<'db> {
    pub const fn from_inner(inner: &'db SqliteConnection) -> Self {
        Self(inner)
    }

    pub const fn into_inner(self) -> &'db SqliteConnection {
        let Self(inner) = self;
        inner
    }
}

impl<'db> From<&'db SqliteConnection> for Connection<'db> {
    fn from(inner: &'db SqliteConnection) -> Self {
        Self::from_inner(inner)
    }
}

impl<'db> From<Connection<'db>> for &'db SqliteConnection {
    fn from(from: Connection<'db>) -> Self {
        from.into_inner()
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
