// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_repo_sqlite::{DbConnection, initialize_connection, initialize_database, run_migrations};
use aoide_storage_sqlite::connection::{
    Config as ConnectionConfig,
    pool::{ConnectionPool, create_connection_pool, get_pooled_connection},
};

use crate::{Error, Result};

pub fn migrate_schema(connection: &mut DbConnection) -> Result<()> {
    // Some migrations require to temporarily disable foreign key constraints.
    // This cannot be accomplished in a migrations script that is executed
    // within a transaction scope.
    // See also: https://www.sqlite.org/pragma.html#pragma_foreign_keys
    connection.set_foreign_keys_enabled(false)?;
    for migration_version in run_migrations(connection)
        .map_err(|err| anyhow::anyhow!(err))
        .map_err(Error::DatabaseMigration)?
    {
        log::info!("Applied migration '{migration_version}'");
    }
    // Re-enable foreign key constraints.
    connection.set_foreign_keys_enabled(true)?;
    Ok(())
}

/// Create a connection pool, initialize the database, and optionally
/// apply pending schema migrations.
pub fn provision_connection_pool(
    config: &ConnectionConfig,
    apply_pending_migrations: bool,
) -> anyhow::Result<ConnectionPool> {
    log::info!(
        "Provisioning SQLite database: {storage}",
        storage = config.storage,
    );

    // The maximum size of the pool defines the maximum number of
    // allowed readers while writers require exclusive access.
    let max_size = config.pool.max_size;
    log::info!("Creating connection pool of max. size {max_size}");
    let connection_pool = create_connection_pool(&config.storage, max_size, initialize_connection)?;

    log::info!("Initializing database");
    initialize_database(&mut *get_pooled_connection(&connection_pool)?)?;

    if apply_pending_migrations {
        log::info!("Migrating database schema");
        migrate_schema(&mut *get_pooled_connection(&connection_pool)?)?;
    }

    Ok(connection_pool)
}
