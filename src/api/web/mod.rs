// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

use actix::prelude::*;

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

use failure::Error;

use r2d2::{Pool, PooledConnection};

pub mod albums;

pub mod collections;

pub mod tracks;

pub type SqliteConnectionManager = ConnectionManager<SqliteConnection>;
pub type SqliteConnectionPool = Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = PooledConnection<SqliteConnectionManager>;

pub struct SqliteExecutor {
    connection_pool: SqliteConnectionPool,
}

impl SqliteExecutor {
    pub fn new(connection_pool: SqliteConnectionPool) -> Self {
        Self { connection_pool }
    }

    pub fn pooled_connection(&self) -> Result<SqlitePooledConnection, Error> {
        self.connection_pool.get().map_err(|e| e.into())
    }
}

impl Actor for SqliteExecutor {
    type Context = SyncContext<Self>;
}

pub struct AppState {
    pub executor: Addr<Syn, SqliteExecutor>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithTokensQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<String>,
}

impl WithTokensQueryParams {
    pub fn try_with_token(&self, with_token: &str) -> bool {
        match self.with {
            Some(ref with) => with.split(',').any(|token| token == with_token),
            None => false,
        }
    }
}
