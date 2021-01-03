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
    collection::{Collection, Entity as CollectionEntity},
    entity::EntityHeader,
    media,
    util::clock::DateTime,
};

use aoide_repo::collection::{EntityRepo as _, RecordId as CollectionId};

use media::Artwork;

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

    fn resolve_record_ids_by_uri_predicate<'s>(
        &self,
        uri_predicate: StringPredicateBorrowed<'s>,
    ) -> RepoResult<Vec<RecordId>> {
        crate::Connection::new(&self.db)
            .resolve_media_source_ids_by_uri_predicate(self.collection_id, uri_predicate)
    }
}

#[test]
fn insert_media_source() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let created_source = media::Source {
        collected_at: DateTime::now_local(),
        synchronized_at: Some(DateTime::now_utc()),
        uri: "file:///home/test/file.mp3".to_string(),
        content_type: "audio/mpeg".to_string(),
        content_digest: None,
        content: AudioContent {
            duration: Some(DurationMs(543.0)),
            ..Default::default()
        }
        .into(),
        artwork: Artwork {
            media_type: Some("image/jpeg".to_string()),
            digest: Some(vec![0, 1, 2, 3, 4, 3, 2, 1, 0]),
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
fn filter_by_uri_predicate_case_sensitive() -> anyhow::Result<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let collection_id = fixture.collection_id;

    let file_lowercase = media::Source {
        collected_at: DateTime::now_local(),
        synchronized_at: Some(DateTime::now_utc()),
        uri: "file:///home/file.mp3".to_string(),
        content_type: "audio/mpeg".to_string(),
        content_digest: None,
        content: AudioContent {
            duration: Some(DurationMs(1.0)),
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
        uri: "file:///Home/File.mp3".to_string(),
        content_type: "audio/mpeg".to_string(),
        content_digest: None,
        content: AudioContent {
            duration: Some(DurationMs(1.0)),
            ..Default::default()
        }
        .into(),
        artwork: Default::default(),
    };
    let header_uppercase =
        db.insert_media_source(DateTime::now_utc(), collection_id, &file_uppercase)?;

    assert_eq!(
        vec![header_lowercase.id],
        fixture.resolve_record_ids_by_uri_predicate(StringPredicateBorrowed::Equals(
            &file_lowercase.uri
        ))?
    );
    assert!(fixture
        .resolve_record_ids_by_uri_predicate(StringPredicateBorrowed::Equals(
            &file_lowercase.uri.to_uppercase()
        ))?
        .is_empty());

    assert_eq!(
        vec![header_uppercase.id],
        fixture.resolve_record_ids_by_uri_predicate(StringPredicateBorrowed::Equals(
            &file_uppercase.uri
        ))?
    );
    assert_eq!(
        vec![header_lowercase.id],
        fixture.resolve_record_ids_by_uri_predicate(StringPredicateBorrowed::Equals(
            &file_uppercase.uri.to_lowercase()
        ))?
    );

    // Prefix filtering is case-insensitive
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_uri_predicate(StringPredicateBorrowed::StartsWith(
            &file_lowercase.uri
        ))?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_uri_predicate(StringPredicateBorrowed::StartsWith(
            &file_uppercase.uri
        ))?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_uri_predicate(StringPredicateBorrowed::StartsWith(
            "file:///home"
        ))?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_uri_predicate(StringPredicateBorrowed::StartsWith(
            "file:///Home"
        ))?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture
            .resolve_record_ids_by_uri_predicate(StringPredicateBorrowed::StartsWith("file:///"))?
    );
    assert!(fixture
        .resolve_record_ids_by_uri_predicate(StringPredicateBorrowed::StartsWith(
            "file:///%home" // LIKE wildcard in predicate string
        ))?
        .is_empty());
    assert!(fixture
        .resolve_record_ids_by_uri_predicate(StringPredicateBorrowed::StartsWith(
            "file:/\\/home" // backslash in predicate string
        ))?
        .is_empty());

    Ok(())
}
