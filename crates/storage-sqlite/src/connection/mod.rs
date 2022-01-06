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

use diesel::r2d2;

use crate::Result;

pub type ConnectionManager = r2d2::ConnectionManager<diesel::SqliteConnection>;

pub type ConnectionPool = r2d2::Pool<ConnectionManager>;

pub type PooledConnection = r2d2::PooledConnection<ConnectionManager>;

#[cfg(feature = "with-tokio")]
pub mod gatekeeper;

pub fn create_connection_pool(database_url: &str, max_size: u32) -> Result<ConnectionPool> {
    let manager = ConnectionManager::new(database_url);
    let pool = ConnectionPool::builder()
        .max_size(max_size)
        .build(manager)?;
    Ok(pool)
}

pub fn get_pooled_connection(pool: &ConnectionPool) -> Result<PooledConnection> {
    pool.get().map_err(Into::into)
}
