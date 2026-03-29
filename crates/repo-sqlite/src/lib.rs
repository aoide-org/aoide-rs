// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// These casts are needed for conversions from/to SQL types.
#![allow(clippy::cast_possible_wrap)]
// Suppress warnings for diesel AsChangeset.
#![allow(clippy::ref_option_ref)]

use std::ops::{Deref, DerefMut};

use anyhow::anyhow;
use diesel::{QueryResult, migration::MigrationVersion, prelude::*, result::Error as DieselError};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness as _, embed_migrations};
use unicase::UniCase;

use aoide_core::EntityUid;
use aoide_repo::RepoError;
use aoide_storage_sqlite::VacuumMode;

mod db;

pub mod repo;

mod util;

pub(crate) use aoide_repo::RecordId as RowId;

const INIT_DB_SQL: &str = include_str!("init_db.sql");

pub(crate) const UNICASE_COLLATION_NAME: &str = "UNICASE";

pub const DEFAULT_VACUUM_MODE: VacuumMode = VacuumMode::Incremental;

pub type DbBackend = diesel::sqlite::Sqlite;
pub type DbConnection = diesel::sqlite::SqliteConnection;

#[expect(missing_debug_implementations)]
pub struct Connection<'db>(&'db mut DbConnection);

impl<'db> Connection<'db> {
    pub fn new(inner: &'db mut DbConnection) -> Self {
        Self(inner)
    }
}

impl<'db> From<&'db mut DbConnection> for Connection<'db> {
    fn from(inner: &'db mut DbConnection) -> Self {
        Self::new(inner)
    }
}

impl AsRef<DbConnection> for Connection<'_> {
    fn as_ref(&self) -> &DbConnection {
        self.0
    }
}

impl AsMut<DbConnection> for Connection<'_> {
    fn as_mut(&mut self) -> &mut DbConnection {
        self.0
    }
}

impl Deref for Connection<'_> {
    type Target = DbConnection;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for Connection<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

pub(crate) fn repo_error(err: DieselError) -> RepoError {
    match err {
        DieselError::NotFound => RepoError::NotFound,
        err => RepoError::Other(err.into()),
    }
}

/// Configure the database engine
///
/// The implementation of the repositories and use cases relies on a proper
/// configuration of the database engine like the behavior, e.g. recursive
/// cascading deletes.
///
/// Some values like the text encoding can only be changed once after the
/// database has initially been created.
pub fn initialize_database(connection: &mut DbConnection) -> QueryResult<()> {
    diesel::sql_query(INIT_DB_SQL).execute(connection)?;

    // FIXME: How to use this collation for all LIKE queries instead of
    // the default NOCASE comparison?
    //
    // The built-in LIKE operator doesn't support case-insensitive matching with
    // custom collations beyond ASCII. You need to overload it separately using
    // sqlite3_create_function(). Both the 2-arg and 3-arg (escaped) versions of
    // LIKE are affected and need to be overloaded separately!
    //
    // Currently, "Beyonce" doesn't match "Beyoncé" and "Ä" doesn't match "ä".
    connection.register_collation(UNICASE_COLLATION_NAME, |lhs, rhs| {
        UniCase::new(lhs).cmp(&UniCase::new(rhs))
    })?;

    Ok(())
}

const EMBEDDED_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

const ENTITY_UID_FROM_ULID_TO_UUID_V7_MIGRATION_VERSION: &str = "0025";

pub fn run_migrations(connection: &mut DbConnection) -> anyhow::Result<Vec<MigrationVersion<'_>>> {
    let pending = connection
        .pending_migrations(EMBEDDED_MIGRATIONS)
        .map_err(|err| anyhow!("pending migrations: {err:#}"))?;
    let Some(entity_uid_from_ulid_to_uuid_v7_index) =
        pending.iter().enumerate().find_map(|(index, migration)| {
            (migration.name().version().to_string().as_str()
                == ENTITY_UID_FROM_ULID_TO_UUID_V7_MIGRATION_VERSION)
                .then_some(index)
        })
    else {
        // Run all pending migrations.
        return connection
            .run_migrations(&pending)
            .map_err(|err| anyhow!("migration failed: {err:#}"));
    };
    let versions_before = connection
        .run_migrations(&pending[..entity_uid_from_ulid_to_uuid_v7_index])
        .map_err(|err| anyhow!("migration failed: {err:#}"))?
        .into_iter()
        .map(|version| version.as_owned())
        .collect::<Vec<_>>();
    log::info!("Migrating entity UIDs from ULID to UUID v7");
    connection.exclusive_transaction::<_, anyhow::Error, _>(|conn| {
        let collection_count = migrate_collection_entity_uid_from_ulid_to_uuid_v7(conn)?;
        log::info!("Migrated {collection_count} collection UID(s) from ULID to UUID v7");
        let playlist_count = migrate_playlist_entity_uid_from_ulid_to_uuid_v7(conn)?;
        log::info!("Migrated {playlist_count} playlist UID(s) from ULID to UUID v7");
        let track_count = migrate_track_entity_uid_from_ulid_to_uuid_v7(conn)?;
        log::info!("Migrated {track_count} track UID(s) from ULID to UUID v7");
        Ok(())
    })?;
    // Run the remaining migrations, including ENTITY_UID_FROM_ULID_TO_UUID_V7_MIGRATION_VERSION.
    let versions_after = if entity_uid_from_ulid_to_uuid_v7_index < pending.len() {
        connection
            .run_migrations(&pending[entity_uid_from_ulid_to_uuid_v7_index..])
            .map_err(|err| anyhow!("migration failed: {err:#}"))?
            .into_iter()
            .map(|version| version.as_owned())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    Ok(versions_before.into_iter().chain(versions_after).collect())
}

fn migrate_collection_entity_uid_from_ulid_to_uuid_v7(
    conn: &mut SqliteConnection,
) -> anyhow::Result<usize> {
    let row_ids = crate::db::collection::schema::collection::table
        .select((crate::db::collection::schema::collection::row_id,))
        .get_results::<(RowId,)>(conn)?;
    for (row_id,) in &row_ids {
        let rows_affected = diesel::update(
            crate::db::collection::schema::collection::table
                .filter(crate::db::collection::schema::collection::row_id.eq(row_id)),
        )
        .set(
            crate::db::collection::schema::collection::entity_uid
                .eq(EntityUid::random().to_string()),
        )
        .execute(conn)?;
        debug_assert_eq!(rows_affected, 1);
    }
    Ok(row_ids.len())
}

fn migrate_playlist_entity_uid_from_ulid_to_uuid_v7(
    conn: &mut SqliteConnection,
) -> anyhow::Result<usize> {
    let row_ids = crate::db::playlist::schema::playlist::table
        .select((crate::db::playlist::schema::playlist::row_id,))
        .get_results::<(RowId,)>(conn)?;
    for (row_id,) in &row_ids {
        let rows_affected = diesel::update(
            crate::db::playlist::schema::playlist::table
                .filter(crate::db::playlist::schema::playlist::row_id.eq(row_id)),
        )
        .set(crate::db::playlist::schema::playlist::entity_uid.eq(EntityUid::random().to_string()))
        .execute(conn)?;
        debug_assert_eq!(rows_affected, 1);
    }
    Ok(row_ids.len())
}

fn migrate_track_entity_uid_from_ulid_to_uuid_v7(
    conn: &mut SqliteConnection,
) -> anyhow::Result<usize> {
    let row_ids = crate::db::track::schema::track::table
        .select((crate::db::track::schema::track::row_id,))
        .get_results::<(RowId,)>(conn)?;
    for (row_id,) in &row_ids {
        let rows_affected = diesel::update(
            crate::db::track::schema::track::table
                .filter(crate::db::track::schema::track::row_id.eq(row_id)),
        )
        .set(crate::db::track::schema::track::entity_uid.eq(EntityUid::random().to_string()))
        .execute(conn)?;
        debug_assert_eq!(rows_affected, 1);
    }
    Ok(row_ids.len())
}

#[cfg(test)]
pub mod tests {
    use anyhow::anyhow;
    use diesel::Connection as _;

    use super::{DbConnection, initialize_database, repo_error, run_migrations};

    pub type TestResult<T> = anyhow::Result<T>;

    #[expect(clippy::missing_panics_doc)] // Never panics
    pub fn establish_connection() -> TestResult<DbConnection> {
        let mut connection =
            DbConnection::establish(":memory:").expect("in-memory database connection");
        initialize_database(&mut connection).map_err(repo_error)?;
        run_migrations(&mut connection).map_err(|err| anyhow!(err))?;
        Ok(connection)
    }
}
