// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
use test_log::test;

use aoide_core::{
    collection::{Collection, Entity as CollectionEntity, MediaSourceConfig},
    entity::EntityHeaderTyped,
    media::content::ContentPathConfig,
    util::{clock::DateTime, url::BaseUrl},
};

use aoide_repo::{
    collection::{EntityRepo as _, RecordId as CollectionId},
    media::DigestBytes,
};

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
            media_source_config: MediaSourceConfig {
                content_path: ContentPathConfig::VirtualFilePath {
                    root_url: BaseUrl::parse_strict("file:///").unwrap(),
                },
            },
        };
        let db = establish_connection()?;
        let collection_entity =
            CollectionEntity::new(EntityHeaderTyped::initial_random(), collection);
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
    let path = ContentPath::new("file:///test/".to_owned());
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
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let updated_at = DateTime::now_utc();
    let collection_id = fixture.collection_id;
    let path = ContentPath::new("file:///test/".to_owned());
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
