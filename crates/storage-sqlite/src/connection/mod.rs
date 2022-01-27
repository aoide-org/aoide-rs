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

use std::num::NonZeroU32;

use diesel::{r2d2, Connection as _};

use crate::Result;

pub type ConnectionManager = r2d2::ConnectionManager<diesel::SqliteConnection>;

pub type ConnectionPool = r2d2::Pool<ConnectionManager>;

pub type PooledConnection = r2d2::PooledConnection<ConnectionManager>;

#[cfg(feature = "with-tokio")]
pub mod gatekeeper;

pub fn create_connection_pool(connection: &str, max_size: NonZeroU32) -> Result<ConnectionPool> {
    // Establish a test connection before creating the connection pool to fail early.
    // If the given file is inaccesible r2d2 (Diesel 1.4.8) seems to do multiple retries
    // and logs errors instead of simply failing and returning and error immediately.
    // Example file name: connection = ":/tmp/aoide.sqlite"
    let _ = diesel::SqliteConnection::establish(connection)?;
    // The test connection is dropped immediately without using it
    // and missing files should have been created after reaching
    // this point.
    let manager = ConnectionManager::new(connection);
    let pool = ConnectionPool::builder()
        .max_size(max_size.get())
        .build(manager)?;
    Ok(pool)
}

pub fn get_pooled_connection(pool: &ConnectionPool) -> Result<PooledConnection> {
    pool.get().map_err(Into::into)
}
