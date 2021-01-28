// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

use crate::prelude::tests::*;

use aoide_core::{
    collection::{Collection, Entity as CollectionEntity},
    entity::EntityHeader,
    util::clock::DateTime,
};

use aoide_repo::collection::{EntityRepo as _, RecordId as CollectionId};

struct Fixture {
    db: SqliteConnection,
    collection_id: CollectionId,
}

impl Fixture {
    fn new() -> TestResult<Self> {
        let collection = Collection {
            title: "Collection".into(),
            notes: None,
            kind: None,
            color: None,
        };
        let db = establish_connection()?;
        let collection_entity = CollectionEntity::new(EntityHeader::initial_random(), collection);
        let collection_id = crate::Connection::new(&db)
            .insert_collection_entity(DateTime::now_utc(), &collection_entity)?;
        Ok(Self { db, collection_id })
    }
}

#[test]
fn update_entry_digest() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let updated_at = DateTime::now_utc();
    let collection_id = fixture.collection_id;
    let uri = "file:///test";
    let mut digest = EntryDigest::default();

    // -> Added
    assert_eq!(
        UpdateOutcome::Inserted,
        db.media_dir_cache_update_entry_digest(updated_at, collection_id, uri, &digest)?
    );
    assert_eq!(
        EntryStatus::Added,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );
    // Added -> Added
    assert_eq!(
        UpdateOutcome::Skipped,
        db.media_dir_cache_update_entry_digest(updated_at, collection_id, uri, &digest)?
    );
    assert_eq!(
        EntryStatus::Added,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    digest[0] = !digest[0];

    // Added -> Modified
    assert_eq!(
        UpdateOutcome::Updated,
        db.media_dir_cache_update_entry_digest(updated_at, collection_id, uri, &digest)?
    );
    assert_eq!(
        EntryStatus::Modified,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );
    // Modified -> Modified
    assert_eq!(
        UpdateOutcome::Skipped,
        db.media_dir_cache_update_entry_digest(updated_at, collection_id, uri, &digest)?
    );
    assert_eq!(
        EntryStatus::Modified,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    // -> Orphaned
    assert_eq!(
        1,
        db.media_dir_cache_update_entries_status(
            updated_at,
            collection_id,
            uri,
            None,
            EntryStatus::Orphaned
        )?
    );
    assert_eq!(
        EntryStatus::Orphaned,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );
    // Orphaned -> Current (digest unchanged)
    assert_eq!(
        UpdateOutcome::Current,
        db.media_dir_cache_update_entry_digest(updated_at, collection_id, uri, &digest)?
    );
    assert_eq!(
        EntryStatus::Current,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    digest[0] = !digest[0];

    // Orphaned -> Modified (after digest changed)
    assert_eq!(
        UpdateOutcome::Updated,
        db.media_dir_cache_update_entry_digest(updated_at, collection_id, uri, &digest)?
    );
    assert_eq!(
        EntryStatus::Modified,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    // -> Current
    assert_eq!(
        1,
        db.media_dir_cache_update_entries_status(
            updated_at,
            collection_id,
            uri,
            None,
            EntryStatus::Current
        )?
    );
    assert_eq!(
        EntryStatus::Current,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    digest[0] = !digest[0];

    // Current -> Modified (after digest changed)
    assert_eq!(
        UpdateOutcome::Updated,
        db.media_dir_cache_update_entry_digest(updated_at, collection_id, uri, &digest)?
    );
    assert_eq!(
        EntryStatus::Modified,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    // -> Outdated
    assert_eq!(
        1,
        db.media_dir_cache_update_entries_status(
            updated_at,
            collection_id,
            uri,
            None,
            EntryStatus::Outdated
        )?
    );
    assert_eq!(
        EntryStatus::Outdated,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    // Outdated -> Current (digest unchanged)
    assert_eq!(
        UpdateOutcome::Current,
        db.media_dir_cache_update_entry_digest(updated_at, collection_id, uri, &digest)?
    );
    assert_eq!(
        EntryStatus::Current,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    // -> Outdated
    assert_eq!(
        1,
        db.media_dir_cache_update_entries_status(
            updated_at,
            collection_id,
            uri,
            None,
            EntryStatus::Outdated
        )?
    );
    assert_eq!(
        EntryStatus::Outdated,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    digest[0] = !digest[0];

    // Outdated -> Modified (after digest changed)
    assert_eq!(
        UpdateOutcome::Updated,
        db.media_dir_cache_update_entry_digest(updated_at, collection_id, uri, &digest)?
    );
    assert_eq!(
        EntryStatus::Modified,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    Ok(())
}

#[test]
fn reset_entry_status_to_current() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let updated_at = DateTime::now_utc();
    let collection_id = fixture.collection_id;
    let uri = "file:///test";
    let digest = EntryDigest::default();

    let mut other_digest = digest.clone();
    other_digest[0] = !other_digest[0];

    // -> Added
    assert_eq!(
        UpdateOutcome::Inserted,
        db.media_dir_cache_update_entry_digest(updated_at, collection_id, uri, &digest)?
    );
    assert_eq!(
        EntryStatus::Added,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    // Added -> Current
    assert!(!db.media_dir_cache_reset_entry_status_to_current(
        updated_at,
        collection_id,
        uri,
        &other_digest
    )?);
    assert_eq!(
        EntryStatus::Added, // unchanged
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );
    assert!(db.media_dir_cache_reset_entry_status_to_current(
        updated_at,
        collection_id,
        uri,
        &digest
    )?);
    assert_eq!(
        EntryStatus::Current, // updated
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    // -> Modified
    assert_eq!(
        1,
        db.media_dir_cache_update_entries_status(
            updated_at,
            collection_id,
            uri,
            None,
            EntryStatus::Modified
        )?
    );
    assert_eq!(
        EntryStatus::Modified,
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    // Modified -> Current
    assert!(!db.media_dir_cache_reset_entry_status_to_current(
        updated_at,
        collection_id,
        uri,
        &other_digest
    )?);
    assert_eq!(
        EntryStatus::Modified, // unchanged
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );
    assert!(db.media_dir_cache_reset_entry_status_to_current(
        updated_at,
        collection_id,
        uri,
        &digest
    )?);
    assert_eq!(
        EntryStatus::Current, // updated
        db.media_dir_cache_load_entry_status_by_uri(collection_id, uri)?
    );

    Ok(())
}
