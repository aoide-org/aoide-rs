// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

use crate::prelude::tests::*;

use test_log::test;

use aoide_core::{entity::EntityHeaderTyped, media, util::url::BaseUrl};

struct Fixture {
    db: DbConnection,
}

impl Fixture {
    pub(super) fn new() -> TestResult<Self> {
        let db = establish_connection()?;
        Ok(Self { db })
    }
}

fn create_collection(repo: &mut dyn EntityRepo, collection: Collection) -> RepoResult<Entity> {
    let entity = Entity::new(EntityHeaderTyped::initial_random(), collection);
    repo.insert_collection_entity(DateTime::now_utc(), &entity)
        .and(Ok(entity))
}

#[test]
fn insert_collection() -> TestResult<()> {
    let mut fixture = Fixture::new()?;
    let mut db = crate::Connection::new(&mut fixture.db);

    let entity = create_collection(
        &mut db,
        Collection {
            title: "Test Collection".into(),
            notes: Some("Some personal notes".into()),
            kind: None,
            color: None,
            media_source_config: MediaSourceConfig {
                content_path: media::content::ContentPathConfig::VirtualFilePath {
                    root_url: BaseUrl::parse_strict("file:///").unwrap(),
                },
            },
        },
    )
    .unwrap();
    println!("Created entity: {entity:?}");
    Ok(())
}

#[test]
fn update_collection() -> TestResult<()> {
    let mut fixture = Fixture::new()?;
    let mut db = crate::Connection::new(&mut fixture.db);

    let mut entity = create_collection(
        &mut db,
        Collection {
            title: "Test Collection".into(),
            notes: Some("Description".into()),
            kind: None,
            color: None,
            media_source_config: MediaSourceConfig {
                content_path: media::content::ContentPathConfig::VirtualFilePath {
                    root_url: BaseUrl::parse_strict("file:///").unwrap(),
                },
            },
        },
    )?;
    let id = db.resolve_collection_id(&entity.hdr.uid)?;

    // Bump revision number for testing
    let outdated_rev = entity.hdr.rev;
    entity.hdr.rev = outdated_rev.next().unwrap();
    db.update_collection_entity(id, DateTime::now_utc(), &entity)?;
    assert_eq!(entity, db.load_collection_entity(id)?.1);

    // Prepare update
    let mut updated_entity = entity.clone();
    updated_entity.body.title = "Retitled Collection".into();
    assert_ne!(entity, updated_entity);

    // Revision not bumped -> Conflict
    assert!(matches!(
        db.update_collection_entity_revision(DateTime::now_utc(), &updated_entity),
        Err(RepoError::Conflict),
    ));
    // Unchanged
    assert_eq!(entity, db.load_collection_entity(id)?.1);

    // Revision bumped twice -> Conflict
    updated_entity.raw.hdr = updated_entity
        .raw
        .hdr
        .next_rev()
        .unwrap()
        .next_rev()
        .unwrap();
    assert!(matches!(
        db.update_collection_entity_revision(DateTime::now_utc(), &updated_entity),
        Err(RepoError::Conflict),
    ));
    // Unchanged
    assert_eq!(entity, db.load_collection_entity(id)?.1);

    // Revision bumped once -> Success
    updated_entity.raw.hdr = updated_entity.raw.hdr.prev_rev().unwrap();
    db.update_collection_entity_revision(DateTime::now_local_or_utc(), &updated_entity)?;
    // Updated
    assert_eq!(updated_entity, db.load_collection_entity(id)?.1);

    // Revert update
    db.update_collection_entity(id, DateTime::now_utc(), &entity)?;
    assert_eq!(entity, db.load_collection_entity(id)?.1);

    Ok(())
}

#[test]
fn purge_collection() -> TestResult<()> {
    let mut fixture = Fixture::new()?;
    let mut db = crate::Connection::new(&mut fixture.db);

    let entity = create_collection(
        &mut db,
        Collection {
            title: "Test Collection".into(),
            notes: None,
            kind: None,
            color: None,
            media_source_config: MediaSourceConfig {
                content_path: media::content::ContentPathConfig::VirtualFilePath {
                    root_url: BaseUrl::parse_strict("file:///").unwrap(),
                },
            },
        },
    )
    .unwrap();
    println!("Created entity: {entity:?}");
    let uid = &entity.hdr.uid;
    let id = db.resolve_collection_id(uid)?;
    db.purge_collection_entity(id)?;
    println!("Removed entity: {uid}");
    Ok(())
}
