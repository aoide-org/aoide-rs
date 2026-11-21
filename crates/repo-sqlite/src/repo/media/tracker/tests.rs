// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use test_log::test;

use aoide_core::{
    Collection, CollectionEntity, CollectionHeader,
    media::{self, content::AudioContentMetadata},
    util::clock::{OffsetDateTimeMs, UtcDateTimeMs},
};
use aoide_repo::{
    CollectionId, collection::EntityRepo as _, media::DigestBytes,
    media::source::CollectionRepo as _,
};

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
#[expect(clippy::too_many_lines, reason = "TODO")]
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

#[test]
fn filter_untracked_content_paths() -> anyhow::Result<()> {
    let mut fixture = Fixture::new()?;
    let mut db = crate::Connection::new(&mut fixture.db);
    let collection_id = fixture.collection_id;
    let updated_at = UtcDateTimeMs::now();

    let dir_path = ContentPath::from("test/");
    db.media_tracker_update_directory_digest(
        updated_at,
        collection_id,
        &dir_path,
        &DigestBytes::default(),
    )?;
    let mut insert_media_source = |path: &str| {
        let media_source = media::Source {
            collected_at: OffsetDateTimeMs::now_local(),
            content: media::Content {
                link: media::content::ContentLink {
                    path: path.to_owned().into(),
                    rev: None,
                },
                r#type: "audio/mpeg".parse().unwrap(),
                metadata_flags: Default::default(),
                metadata: AudioContentMetadata::default().into(),
                digest: None,
            },
            artwork: Default::default(),
        };
        db.insert_media_source(collection_id, updated_at, &media_source)
            .map(|header| header.id)
    };
    let tracked_a = insert_media_source("test/a.mp3")?;
    let tracked_b = insert_media_source("test/b.mp3")?;
    // A media source that is not tracked
    insert_media_source("test/c.mp3")?;
    db.media_tracker_replace_directory_sources(collection_id, &dir_path, &[tracked_a, tracked_b])?;

    let content_paths = ["test/c.mp3", "test/a.mp3", "test/d.mp3", "test/b.mp3"]
        .into_iter()
        .map(|path| ContentPath::from(path.to_owned()))
        .collect();
    let untracked =
        db.media_tracker_filter_untracked_content_paths(collection_id, content_paths)?;
    assert_eq!(
        vec![
            ContentPath::from("test/c.mp3".to_owned()),
            ContentPath::from("test/d.mp3".to_owned()),
        ],
        untracked
    );

    Ok(())
}
