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
    audio::{AudioContent, DurationMs},
    collection::{Collection, Entity as CollectionEntity, MediaSourceConfig},
    entity::EntityHeader,
    media::{self, SourcePath, SourcePathKind},
    util::clock::DateTime,
};

use aoide_repo::collection::{EntityRepo as _, RecordId as CollectionId};

use media::{Artwork, ContentMetadataFlags};

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
                path_kind: SourcePathKind::VirtualFilePath,
                base_url: None,
            },
        };
        let db = establish_connection()?;
        let collection_entity = CollectionEntity::new(EntityHeader::initial_random(), collection);
        let collection_id = crate::Connection::new(&db)
            .insert_collection_entity(DateTime::now_utc(), &collection_entity)?;
        Ok(Self { db, collection_id })
    }

    fn resolve_record_ids_by_path_predicate<'s>(
        &self,
        path_predicate: StringPredicateBorrowed<'s>,
    ) -> RepoResult<Vec<RecordId>> {
        crate::Connection::new(&self.db)
            .resolve_media_source_ids_by_path_predicate(self.collection_id, path_predicate)
    }
}

#[test]
fn insert_media_source() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let created_source = media::Source {
        collected_at: DateTime::now_local(),
        synchronized_at: Some(DateTime::now_utc()),
        path: SourcePath::new("file:///home/test/file.mp3".to_owned()),
        content_type: "audio/mpeg".to_owned(),
        content_digest: None,
        content_metadata_flags: Default::default(),
        content: AudioContent {
            duration: Some(DurationMs::from_inner(543.0)),
            ..Default::default()
        }
        .into(),
        artwork: Artwork {
            media_type: Some("image/jpeg".to_owned()),
            digest: Some([128; 32]),
            thumbnail: Some([127; 4 * 4 * 3]),
            ..Default::default()
        },
    };
    let created_at = DateTime::now_local();

    let created_header =
        db.insert_media_source(created_at, fixture.collection_id, &created_source)?;
    assert_eq!(created_at, created_header.created_at);
    assert_eq!(created_at, created_header.updated_at);

    let (loaded_header, loaded_source) = db.load_media_source(created_header.id)?;
    assert_eq!(created_header, loaded_header);
    assert_eq!(created_source, loaded_source);

    Ok(())
}

#[test]
fn filter_by_path_predicate() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let collection_id = fixture.collection_id;

    let file_lowercase = media::Source {
        collected_at: DateTime::now_local(),
        synchronized_at: Some(DateTime::now_utc()),
        path: SourcePath::new("file:///home/file.mp3".to_owned()),
        content_type: "audio/mpeg".to_owned(),
        content_digest: None,
        content_metadata_flags: Default::default(),
        content: AudioContent {
            duration: Some(DurationMs::from_inner(1.0)),
            ..Default::default()
        }
        .into(),
        artwork: Default::default(),
    };
    let header_lowercase =
        db.insert_media_source(DateTime::now_utc(), collection_id, &file_lowercase)?;

    let file_uppercase = media::Source {
        collected_at: DateTime::now_local(),
        synchronized_at: Some(DateTime::now_utc()),
        path: SourcePath::new("file:///Home/File.mp3".to_owned()),
        content_type: "audio/mpeg".to_owned(),
        content_digest: None,
        content_metadata_flags: ContentMetadataFlags::RELIABLE,
        content: AudioContent {
            duration: Some(DurationMs::from_inner(1.0)),
            ..Default::default()
        }
        .into(),
        artwork: Default::default(),
    };
    let header_uppercase =
        db.insert_media_source(DateTime::now_utc(), collection_id, &file_uppercase)?;

    // Equals is case-sensitive
    assert_eq!(
        vec![header_lowercase.id],
        fixture.resolve_record_ids_by_path_predicate(StringPredicateBorrowed::Equals(
            &file_lowercase.path
        ))?
    );
    assert!(fixture
        .resolve_record_ids_by_path_predicate(StringPredicateBorrowed::Equals(
            &file_lowercase.path.to_uppercase()
        ))?
        .is_empty());

    assert_eq!(
        vec![header_uppercase.id],
        fixture.resolve_record_ids_by_path_predicate(StringPredicateBorrowed::Equals(
            &file_uppercase.path
        ))?
    );
    assert_eq!(
        vec![header_lowercase.id],
        fixture.resolve_record_ids_by_path_predicate(StringPredicateBorrowed::Equals(
            &file_uppercase.path.to_lowercase()
        ))?
    );

    // Prefix is case-sensitive
    assert_eq!(
        vec![header_lowercase.id],
        fixture
            .resolve_record_ids_by_path_predicate(StringPredicateBorrowed::Prefix("file:///ho"))?
    );
    assert_eq!(
        vec![header_uppercase.id],
        fixture
            .resolve_record_ids_by_path_predicate(StringPredicateBorrowed::Prefix("file:///Ho"))?
    );
    assert!(fixture
        .resolve_record_ids_by_path_predicate(StringPredicateBorrowed::Prefix("file:///hO"))?
        .is_empty());
    assert!(fixture
        .resolve_record_ids_by_path_predicate(StringPredicateBorrowed::Prefix("file:///HO"))?
        .is_empty());

    // StartsWith is case-insensitive
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_path_predicate(StringPredicateBorrowed::StartsWith(
            &file_lowercase.path
        ))?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_path_predicate(StringPredicateBorrowed::StartsWith(
            &file_uppercase.path
        ))?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_path_predicate(StringPredicateBorrowed::StartsWith(
            "file:///home"
        ))?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_path_predicate(StringPredicateBorrowed::StartsWith(
            "file:///Home"
        ))?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_path_predicate(StringPredicateBorrowed::StartsWith(
            "file:///"
        ))?
    );
    assert!(fixture
        .resolve_record_ids_by_path_predicate(StringPredicateBorrowed::StartsWith(
            "file:///%home" // LIKE wildcard in predicate string
        ))?
        .is_empty());
    assert!(fixture
        .resolve_record_ids_by_path_predicate(StringPredicateBorrowed::StartsWith(
            "file:/\\/home" // backslash in predicate string
        ))?
        .is_empty());

    Ok(())
}

#[test]
fn relocate_by_path() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let collection_id = fixture.collection_id;

    let file_lowercase = media::Source {
        collected_at: DateTime::now_local(),
        synchronized_at: Some(DateTime::now_utc()),
        path: SourcePath::new("file:///ho''me/file.mp3".to_owned()),
        content_type: "audio/mpeg".to_owned(),
        content_digest: None,
        content_metadata_flags: Default::default(),
        content: AudioContent {
            duration: Some(DurationMs::from_inner(1.0)),
            ..Default::default()
        }
        .into(),
        artwork: Default::default(),
    };
    let header_lowercase =
        db.insert_media_source(DateTime::now_utc(), collection_id, &file_lowercase)?;

    let file_uppercase = media::Source {
        collected_at: DateTime::now_local(),
        synchronized_at: Some(DateTime::now_utc()),
        path: SourcePath::new("file:///Ho''me/File.mp3".to_owned()),
        content_type: "audio/mpeg".to_owned(),
        content_digest: None,
        content_metadata_flags: ContentMetadataFlags::RELIABLE,
        content: AudioContent {
            duration: Some(DurationMs::from_inner(1.0)),
            ..Default::default()
        }
        .into(),
        artwork: Default::default(),
    };
    let header_uppercase =
        db.insert_media_source(DateTime::now_utc(), collection_id, &file_uppercase)?;

    let updated_at = DateTime::now_utc();
    let old_path_prefix = "file:///ho''";
    let new_path_prefix = "file:///h'o''";

    assert_eq!(
        1,
        db.relocate_media_sources_by_path_prefix(
            updated_at,
            collection_id,
            old_path_prefix,
            new_path_prefix
        )?
    );

    assert!(fixture
        .resolve_record_ids_by_path_predicate(StringPredicateBorrowed::Prefix(old_path_prefix))?
        .is_empty());
    assert_eq!(
        vec![header_lowercase.id],
        fixture.resolve_record_ids_by_path_predicate(StringPredicateBorrowed::Prefix(
            new_path_prefix
        ))?
    );
    assert_eq!(
        updated_at,
        db.load_media_source(header_lowercase.id)?.0.updated_at
    );
    assert_eq!(
        vec![header_uppercase.id],
        fixture.resolve_record_ids_by_path_predicate(StringPredicateBorrowed::Prefix(
            "file:///Ho''"
        ))?
    );

    Ok(())
}
