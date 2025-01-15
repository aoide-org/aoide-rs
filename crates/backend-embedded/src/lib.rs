// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use storage::DatabaseConfig;

use crate::storage::provision_database;

pub mod batch;
pub mod collection;
pub mod media;
pub mod playlist;
pub mod storage;
pub mod track;

pub type Error = aoide_usecases_sqlite::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub mod prelude {
    pub use aoide_core::CollectionUid;

    pub use super::{Error, Result};
}

/// Runtime environment for invoking operations
///
/// Holds the database connection.
#[allow(missing_debug_implementations)]
pub struct Environment {
    db_gatekeeper: Gatekeeper,
}

impl Environment {
    /// Set up the runtime environment
    ///
    /// Modifying the database configuration at runtime is not supported.
    pub fn commission(db_config: &DatabaseConfig) -> anyhow::Result<Self> {
        log::info!("Commissioning context");
        let db_gatekeeper = provision_database(db_config)?;
        Ok(Self { db_gatekeeper })
    }

    /// Prepare for shutting down
    ///
    /// Rejects new database requests. Pending requests could still proceed
    /// until finished.
    ///
    /// It is safe to invoke this operation repeatedly.
    pub fn decommission(&self) {
        log::info!("Decommissioning context");
        self.db_gatekeeper.decommission();
    }

    /// The [`Gatekeeper`] for accessing the database
    pub const fn db_gatekeeper(&self) -> &Gatekeeper {
        &self.db_gatekeeper
    }
}
