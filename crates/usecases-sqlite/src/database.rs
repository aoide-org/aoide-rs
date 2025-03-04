// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_repo_sqlite::{DbConnection, run_migrations};

use crate::{Error, Result};

pub fn migrate_schema(connection: &mut DbConnection) -> Result<()> {
    for migration_version in run_migrations(connection)
        .map_err(|err| anyhow::anyhow!(err))
        .map_err(Error::DatabaseMigration)?
    {
        log::info!("Applied migration '{migration_version}'");
    }
    Ok(())
}
