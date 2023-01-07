// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::num::NonZeroU32;

use diesel::{r2d2, Connection as _};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Result;

use super::Storage;

pub type ConnectionManager = r2d2::ConnectionManager<diesel::SqliteConnection>;

pub type ConnectionPool = r2d2::Pool<ConnectionManager>;

pub type PooledConnection = r2d2::PooledConnection<ConnectionManager>;

#[cfg(feature = "tokio")]
pub mod gatekeeper;

pub fn create_connection_pool(storage: &Storage, max_size: NonZeroU32) -> Result<ConnectionPool> {
    let storage = storage.as_ref();
    // Establish a test connection before creating the connection pool to fail early.
    // If the given file is inaccessible r2d2 (Diesel 1.4.8) seems to do multiple retries
    // and logs errors instead of simply failing and returning and error immediately.
    // Example file name: connection = ":/tmp/aoide.sqlite"
    std::mem::drop(diesel::SqliteConnection::establish(storage)?);
    // The test connection is dropped immediately without using it
    // and missing files should have been created after reaching
    // this point.
    let manager = ConnectionManager::new(storage);
    let pool = ConnectionPool::builder()
        .max_size(max_size.get())
        .build(manager)?;
    Ok(pool)
}

pub fn get_pooled_connection(pool: &ConnectionPool) -> Result<PooledConnection> {
    pool.get().map_err(Into::into)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    pub max_size: NonZeroU32,

    #[cfg(feature = "tokio")]
    pub gatekeeper: self::gatekeeper::Config,
}
