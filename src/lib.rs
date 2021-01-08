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

pub mod api;
pub mod usecases;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

#[macro_use]
extern crate diesel_migrations;

use aoide_repo_sqlite::prelude::{Connection as RepoConnection, *};

pub type SqliteConnectionManager = ConnectionManager<SqliteConnection>;
pub type SqliteConnectionPool = Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = PooledConnection<SqliteConnectionManager>;

// Maximum number of concurrent readers, only a single
// concurrent writer.
pub const DB_CONNECTION_POOL_SIZE: u32 = 8;
