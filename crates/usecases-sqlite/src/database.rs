// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::RunQueryDsl as _;

use aoide_repo_sqlite::{DbConnection, run_migrations};

use crate::{Error, Result};

pub fn migrate_schema(connection: &mut DbConnection) -> Result<()> {
    // Some migrations require to temporarily disable foreign key constraints.
    // This cannot be accomplished in a migrations script that is executed
    // within a transaction scope.
    // See also: https://www.sqlite.org/pragma.html#pragma_foreign_keys
    diesel::sql_query("PRAGMA foreign_keys=OFF").execute(connection)?;
    for migration_version in run_migrations(connection)
        .map_err(|err| anyhow::anyhow!(err))
        .map_err(Error::DatabaseMigration)?
    {
        log::info!("Applied migration '{migration_version}'");
    }
    // Re-enable foreign key constraints.
    diesel::sql_query("PRAGMA foreign_keys=ON").execute(connection)?;
    Ok(())
}
