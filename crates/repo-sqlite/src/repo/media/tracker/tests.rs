// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use test_log::test;

use aoide_core::{Collection, CollectionEntity, CollectionHeader, util::clock::UtcDateTimeMs};
use aoide_repo::{CollectionId, collection::EntityRepo as _, media::DigestBytes};

use crate::{DbConnection, repo::tests::vfs_media_source_config, tests::*};

use super::*;

struct Fixture {
    db: DbConnection,
    collection_id: CollectionId,
}

impl Fixture {
    fn new() -> TestResult<Self> {
        let collection = Collection {
            title: "Collection".into(),
            notes: None,
            kind: None,
            color: None,
            media_source_config: vfs_media_source_config(),
        };
        let mut db = establish_connection()?;
        let collection_entity =
            CollectionEntity::new(CollectionHeader::initial_random(), collection);
        let collection_id = crate::Connection::new(&mut db)
            .insert_collection_entity(UtcDateTimeMs::now(), &collection_entity)?;
        Ok(Self { db, collection_id })
    }
}

#[test]
fn update_entry_digest() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let mut db = crate::Connection::new(&mut fixture.db);

    let updated_at = UtcDateTimeMs::now();
    let collection_id = fixture.collection_id;
    let path = ContentPath::from("file:///test/");
    let mut digest = DigestBytes::default();

    // -> Added
    assert_eq!(
        DirUpdateOutcome::Inserted,
        db.media_tracker_update_directory_digest(updated_at, collection_id, &path, &digest)?
    );
    assert_eq!(
        DirTrackingStatus::Added,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );
    // Added -> Added
    assert_eq!(
        DirUpdateOutcome::Skipped,
        db.media_tracker_update_directory_digest(updated_at, collection_id, &path, &digest)?
    );
    assert_eq!(
        DirTrackingStatus::Added,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    digest[0] = !digest[0];

    // Added -> Modified
    assert_eq!(
        DirUpdateOutcome::Updated,
        db.media_tracker_update_directory_digest(updated_at, collection_id, &path, &digest)?
    );
    assert_eq!(
        DirTrackingStatus::Modified,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );
    // Modified -> Modified
    assert_eq!(
        DirUpdateOutcome::Skipped,
        db.media_tracker_update_directory_digest(updated_at, collection_id, &path, &digest)?
    );
    assert_eq!(
        DirTrackingStatus::Modified,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    // -> Orphaned
    assert_eq!(
        1,
        db.media_tracker_update_directories_status(
            updated_at,
            collection_id,
            &path,
            None,
            DirTrackingStatus::Orphaned
        )?
    );
    assert_eq!(
        DirTrackingStatus::Orphaned,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );
    // Orphaned -> Current (digest unchanged)
    assert_eq!(
        DirUpdateOutcome::Current,
        db.media_tracker_update_directory_digest(updated_at, collection_id, &path, &digest)?
    );
    assert_eq!(
        DirTrackingStatus::Current,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    digest[0] = !digest[0];

    // Orphaned -> Modified (after digest changed)
    assert_eq!(
        DirUpdateOutcome::Updated,
        db.media_tracker_update_directory_digest(updated_at, collection_id, &path, &digest)?
    );
    assert_eq!(
        DirTrackingStatus::Modified,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    // -> Current
    assert_eq!(
        1,
        db.media_tracker_update_directories_status(
            updated_at,
            collection_id,
            &path,
            None,
            DirTrackingStatus::Current
        )?
    );
    assert_eq!(
        DirTrackingStatus::Current,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    digest[0] = !digest[0];

    // Current -> Modified (after digest changed)
    assert_eq!(
        DirUpdateOutcome::Updated,
        db.media_tracker_update_directory_digest(updated_at, collection_id, &path, &digest)?
    );
    assert_eq!(
        DirTrackingStatus::Modified,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    // -> Outdated
    assert_eq!(
        1,
        db.media_tracker_update_directories_status(
            updated_at,
            collection_id,
            &path,
            None,
            DirTrackingStatus::Outdated
        )?
    );
    assert_eq!(
        DirTrackingStatus::Outdated,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    // Outdated -> Current (digest unchanged)
    assert_eq!(
        DirUpdateOutcome::Current,
        db.media_tracker_update_directory_digest(updated_at, collection_id, &path, &digest)?
    );
    assert_eq!(
        DirTrackingStatus::Current,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    // -> Outdated
    assert_eq!(
        1,
        db.media_tracker_update_directories_status(
            updated_at,
            collection_id,
            &path,
            None,
            DirTrackingStatus::Outdated
        )?
    );
    assert_eq!(
        DirTrackingStatus::Outdated,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    digest[0] = !digest[0];

    // Outdated -> Modified (after digest changed)
    assert_eq!(
        DirUpdateOutcome::Updated,
        db.media_tracker_update_directory_digest(updated_at, collection_id, &path, &digest)?
    );
    assert_eq!(
        DirTrackingStatus::Modified,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    Ok(())
}

#[test]
fn reset_entry_status_to_current() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let mut db = crate::Connection::new(&mut fixture.db);

    let updated_at = UtcDateTimeMs::now();
    let collection_id = fixture.collection_id;
    let path = ContentPath::from("file:///test/");
    let digest = DigestBytes::default();

    let mut other_digest = digest;
    other_digest[0] = !other_digest[0];

    // -> Added
    assert_eq!(
        DirUpdateOutcome::Inserted,
        db.media_tracker_update_directory_digest(updated_at, collection_id, &path, &digest)?
    );
    assert_eq!(
        DirTrackingStatus::Added,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    // Added -> Current: Rejected
    assert!(!db.media_tracker_confirm_directory(
        updated_at,
        collection_id,
        &path,
        &other_digest,
    )?);
    assert_eq!(
        DirTrackingStatus::Added, // unchanged
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    // Added -> Current: Confirmed
    assert!(db.media_tracker_confirm_directory(updated_at, collection_id, &path, &digest)?);
    assert_eq!(
        DirTrackingStatus::Current,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    // -> Modified
    assert_eq!(
        1,
        db.media_tracker_update_directories_status(
            updated_at,
            collection_id,
            &path,
            None,
            DirTrackingStatus::Modified
        )?
    );
    assert_eq!(
        DirTrackingStatus::Modified,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    // Modified -> Current: Rejected
    assert!(!db.media_tracker_confirm_directory(
        updated_at,
        collection_id,
        &path,
        &other_digest,
    )?);
    assert_eq!(
        DirTrackingStatus::Modified,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    // Modified -> Current: Confirmed
    assert!(db.media_tracker_confirm_directory(updated_at, collection_id, &path, &digest)?);
    assert_eq!(
        DirTrackingStatus::Current,
        db.media_tracker_load_directory_tracking_status(collection_id, &path)?
    );

    Ok(())
}
