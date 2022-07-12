// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// This exception on module level is required for the code generated
// by the embed_migrations macro.
#![allow(clippy::panic_in_result_fn)]

use super::*;

diesel_migrations::embed_migrations!("../repo-sqlite/migrations");

pub fn migrate_schema(connection: &SqliteConnection) -> Result<()> {
    embedded_migrations::run(connection)?;
    Ok(())
}
