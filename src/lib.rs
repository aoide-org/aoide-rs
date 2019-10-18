// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use failure::{Error, Fallible};

pub type SqliteConnectionManager = ConnectionManager<SqliteConnection>;
pub type SqliteConnectionPool = Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = PooledConnection<SqliteConnectionManager>;

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct SqliteExecutor {
    connection_pool: SqliteConnectionPool,
}

impl SqliteExecutor {
    pub fn new(connection_pool: SqliteConnectionPool) -> Self {
        Self { connection_pool }
    }

    pub fn pooled_connection(&self) -> Result<SqlitePooledConnection, Error> {
        self.connection_pool.get().map_err(Into::into)
    }
}
