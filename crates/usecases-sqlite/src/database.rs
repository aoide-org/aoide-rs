// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_repo_sqlite::{run_pending_migrations, DbConnection};

use super::*;

pub fn migrate_schema(connection: &mut DbConnection) -> Result<()> {
    for migration_version in run_pending_migrations(connection)
        .map_err(|err| anyhow::anyhow!(err))
        .map_err(Error::DatabaseMigration)?
    {
        log::info!("Migration succeeded: {migration_version}");
    }
    Ok(())
}
