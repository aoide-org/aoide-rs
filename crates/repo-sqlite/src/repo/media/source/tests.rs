// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use mime::IMAGE_JPEG;
use test_log::test;

use aoide_core::{
    audio::DurationMs,
    media::{
        self,
        artwork::{
            ApicType, Artwork, ArtworkImage, ImageSize, LinkedArtwork, THUMBNAIL_HEIGHT,
            THUMBNAIL_WIDTH,
        },
        content::{
            AudioContentMetadata, ContentLink, ContentMetadataFlags, ContentPath, ContentRevision,
        },
    },
    util::{clock::OffsetDateTimeMs, color::RgbColor},
    Collection, CollectionEntity, CollectionHeader,
};
use aoide_repo::{collection::EntityRepo as _, CollectionId};

use super::*;
use crate::{repo::tests::vfs_media_source_config, tests::*};

struct Fixture {
    collection_id: CollectionId,
}

impl Fixture {
    fn new(db: &mut crate::Connection<'_>) -> TestResult<Self> {
        let collection = Collection {
            title: "Collection".into(),
            notes: None,
            kind: None,
            color: None,
            media_source_config: vfs_media_source_config(),
        };
        let collection_entity =
            CollectionEntity::new(CollectionHeader::initial_random(), collection);
        let created_at = OffsetDateTimeMs::now_utc();
        let collection_id = db.insert_collection_entity(&created_at, &collection_entity)?;
        Ok(Self { collection_id })
    }

    fn resolve_record_ids_by_content_path_predicate(
        &self,
        db: &mut crate::Connection<'_>,
        content_path_predicate: StringPredicate<'_>,
    ) -> RepoResult<Vec<MediaSourceId>> {
        db.resolve_media_source_ids_by_content_path_predicate(
            self.collection_id,
            content_path_predicate,
        )
    }
}

#[test]
fn insert_media_source() -> anyhow::Result<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let fixture = Fixture::new(&mut db)?;

    let created_source = media::Source {
        collected_at: OffsetDateTimeMs::now_local(),
        content: media::Content {
            link: ContentLink {
                path: ContentPath::from("file:///home/test/file.mp3"),
                rev: Some(ContentRevision::new(6)),
            },
            r#type: "audio/mpeg".parse().unwrap(),
            digest: None,
            metadata_flags: Default::default(),
            metadata: AudioContentMetadata {
                duration: Some(DurationMs::new(543.0)),
                ..Default::default()
            }
            .into(),
        },
        artwork: Some(Artwork::Linked(LinkedArtwork {
            uri: "file://folder.jpg".to_owned(),
            image: ArtworkImage {
                media_type: IMAGE_JPEG,
                apic_type: ApicType::CoverFront,
                data_size: 65535,
                image_size: Some(ImageSize {
                    width: 500,
                    height: 600,
                }),
                color: Some(RgbColor::rgb(0xf0, 0xf0, 0xf0)),
                digest: Some([128; 32]),
                thumbnail: Some([127; (THUMBNAIL_WIDTH * THUMBNAIL_HEIGHT * 3) as _]),
            },
        })),
    };
    let created_at = OffsetDateTimeMs::now_local();

    let created_header =
        db.insert_media_source(fixture.collection_id, created_at.clone(), &created_source)?;
    assert_eq!(created_at, created_header.created_at);
    assert_eq!(created_at, created_header.updated_at);

    let (loaded_header, loaded_source) = db.load_media_source(created_header.id)?;
    assert_eq!(created_header, loaded_header);
    assert_eq!(created_source, loaded_source);

    Ok(())
}

#[test]
fn filter_by_content_path_predicate() -> anyhow::Result<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let fixture = Fixture::new(&mut db)?;

    let collection_id = fixture.collection_id;

    let file_lowercase = media::Source {
        collected_at: OffsetDateTimeMs::now_local(),
        content: media::Content {
            link: ContentLink {
                path: ContentPath::from("file:///home/file.mp3"),
                rev: None,
            },
            r#type: "audio/mpeg".parse().unwrap(),
            digest: None,
            metadata_flags: Default::default(),
            metadata: AudioContentMetadata {
                duration: Some(DurationMs::new(1.0)),
                ..Default::default()
            }
            .into(),
        },
        artwork: Default::default(),
    };
    let header_lowercase =
        db.insert_media_source(collection_id, OffsetDateTimeMs::now_utc(), &file_lowercase)?;

    let file_uppercase = media::Source {
        collected_at: OffsetDateTimeMs::now_local(),
        content: media::Content {
            link: ContentLink {
                path: ContentPath::from("file:///Home/File.mp3"),
                rev: None,
            },
            r#type: "audio/mpeg".parse().unwrap(),
            digest: None,
            metadata_flags: ContentMetadataFlags::RELIABLE,
            metadata: AudioContentMetadata {
                duration: Some(DurationMs::new(1.0)),
                ..Default::default()
            }
            .into(),
        },
        artwork: Default::default(),
    };
    let header_uppercase =
        db.insert_media_source(collection_id, OffsetDateTimeMs::now_utc(), &file_uppercase)?;

    // Equals is case-sensitive
    assert_eq!(
        vec![header_lowercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::Equals(file_lowercase.content.link.path.to_borrowed().into_inner())
        )?
    );
    assert!(fixture
        .resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::Equals(
                file_lowercase
                    .content
                    .link
                    .path
                    .as_str()
                    .to_uppercase()
                    .into()
            )
        )?
        .is_empty());

    assert_eq!(
        vec![header_uppercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::Equals(file_uppercase.content.link.path.to_borrowed().into_inner())
        )?
    );
    assert_eq!(
        vec![header_lowercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::Equals(
                file_uppercase
                    .content
                    .link
                    .path
                    .as_str()
                    .to_lowercase()
                    .into()
            )
        )?
    );

    // Prefix is case-sensitive
    assert_eq!(
        vec![header_lowercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::Prefix("file:///ho".into())
        )?
    );
    assert_eq!(
        vec![header_uppercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::Prefix("file:///Ho".into())
        )?
    );
    assert!(fixture
        .resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::Prefix("file:///hO".into())
        )?
        .is_empty());
    assert!(fixture
        .resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::Prefix("file:///HO".into())
        )?
        .is_empty());

    // StartsWith is case-insensitive
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::StartsWith(
                file_lowercase.content.link.path.to_borrowed().into_inner()
            )
        )?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::StartsWith(
                file_uppercase.content.link.path.to_borrowed().into_inner()
            )
        )?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::StartsWith("file:///home".into())
        )?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::StartsWith("file:///Home".into())
        )?
    );
    assert_eq!(
        vec![header_lowercase.id, header_uppercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::StartsWith("file:///".into())
        )?
    );
    assert!(fixture
        .resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::StartsWith(
                "file:///%home".into() // LIKE wildcard in predicate string
            )
        )?
        .is_empty());
    assert!(fixture
        .resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::StartsWith(
            "file:/\\/home".into() // backslash in predicate string
        )
        )?
        .is_empty());

    Ok(())
}

#[test]
fn relocate_by_content_path() -> anyhow::Result<()> {
    let mut db = establish_connection()?;
    let mut db = crate::Connection::new(&mut db);
    let fixture = Fixture::new(&mut db)?;

    let collection_id = fixture.collection_id;

    let file_lowercase = media::Source {
        collected_at: OffsetDateTimeMs::now_local(),
        content: media::Content {
            link: ContentLink {
                path: ContentPath::from("file:///ho''me/file.mp3"),
                rev: None,
            },
            r#type: "audio/mpeg".parse().unwrap(),
            digest: None,
            metadata_flags: Default::default(),
            metadata: AudioContentMetadata {
                duration: Some(DurationMs::new(1.0)),
                ..Default::default()
            }
            .into(),
        },
        artwork: Default::default(),
    };
    let header_lowercase =
        db.insert_media_source(collection_id, OffsetDateTimeMs::now_utc(), &file_lowercase)?;

    let file_uppercase = media::Source {
        collected_at: OffsetDateTimeMs::now_local(),
        content: media::Content {
            link: ContentLink {
                path: ContentPath::from("file:///Ho''me/File.mp3"),
                rev: None,
            },
            r#type: "audio/mpeg".parse().unwrap(),
            digest: None,
            metadata_flags: ContentMetadataFlags::RELIABLE,
            metadata: AudioContentMetadata {
                duration: Some(DurationMs::new(1.0)),
                ..Default::default()
            }
            .into(),
        },
        artwork: Default::default(),
    };
    let header_uppercase =
        db.insert_media_source(collection_id, OffsetDateTimeMs::now_utc(), &file_uppercase)?;

    let updated_at = OffsetDateTimeMs::now_utc();
    let old_path_prefix = ContentPath::from("file:///ho''");
    let new_path_prefix = ContentPath::from("file:///h'o''");

    assert_eq!(
        1,
        db.relocate_media_sources_by_content_path_prefix(
            collection_id,
            &updated_at,
            &old_path_prefix,
            &new_path_prefix
        )?
    );

    assert!(fixture
        .resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::Prefix(old_path_prefix.to_borrowed().into_inner())
        )?
        .is_empty());
    assert_eq!(
        vec![header_lowercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::Prefix(new_path_prefix.to_borrowed().into_inner())
        )?
    );
    assert_eq!(
        updated_at,
        db.load_media_source(header_lowercase.id)?.0.updated_at
    );
    assert_eq!(
        vec![header_uppercase.id],
        fixture.resolve_record_ids_by_content_path_predicate(
            &mut db,
            StringPredicate::Prefix("file:///Ho''".into())
        )?
    );

    Ok(())
}
