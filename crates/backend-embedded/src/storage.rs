// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_storage_sqlite::connection::{Config as ConnectionConfig, pool::gatekeeper::Gatekeeper};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DatabaseSchemaMigrationMode {
    DontTouch,
    #[default]
    ApplyPending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DatabaseConfig {
    pub connection: ConnectionConfig,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub migrate_schema: Option<DatabaseSchemaMigrationMode>,
}

pub fn provision_database(config: &DatabaseConfig) -> anyhow::Result<Gatekeeper> {
    let DatabaseConfig {
        connection,
        migrate_schema,
    } = config;
    let apply_pending_migrations = match migrate_schema.unwrap_or_default() {
        DatabaseSchemaMigrationMode::DontTouch => false,
        DatabaseSchemaMigrationMode::ApplyPending => true,
    };
    let connection_pool = aoide_usecases_sqlite::database::provision_connection_pool(
        connection,
        apply_pending_migrations,
    )?;
    Ok(Gatekeeper::new(connection_pool, connection.pool.gatekeeper))
}
