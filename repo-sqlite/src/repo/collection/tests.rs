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

use aoide_core::{entity::EntityHeader, media};

struct Fixture {
    db: SqliteConnection,
}

impl Fixture {
    pub fn new() -> TestResult<Self> {
        let db = establish_connection()?;
        Ok(Self { db })
    }
}

fn create_collection(repo: &dyn EntityRepo, collection: Collection) -> RepoResult<Entity> {
    let entity = Entity::new(EntityHeader::initial_random(), collection);
    repo.insert_collection_entity(DateTime::now_utc(), &entity)
        .and(Ok(entity))
}

#[test]
fn insert_collection() -> TestResult<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let entity = create_collection(
        &db,
        Collection {
            title: "Test Collection".into(),
            notes: Some("Some personal notes".into()),
            kind: None,
            color: None,
            media_source_config: MediaSourceConfig {
                path_kind: media::SourcePathKind::VirtualFilePath,
                base_url: None,
            },
        },
    )
    .unwrap();
    println!("Created entity: {:?}", entity);
    Ok(())
}

#[test]
fn update_collection() -> TestResult<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let mut entity = create_collection(
        &db,
        Collection {
            title: "Test Collection".into(),
            notes: Some("Description".into()),
            kind: None,
            color: None,
            media_source_config: MediaSourceConfig {
                path_kind: media::SourcePathKind::VirtualFilePath,
                base_url: None,
            },
        },
    )?;
    let id = db.resolve_collection_id(&entity.hdr.uid)?;

    // Bump revision number for testing
    let outdated_rev = entity.hdr.rev;
    entity.hdr.rev = outdated_rev.next();
    db.update_collection_entity(id, DateTime::now_utc(), &entity)?;
    assert_eq!(entity, db.load_collection_entity(id)?.1);

    // Prepare update
    let mut updated_entity = entity.clone();
    updated_entity.hdr.rev = updated_entity.hdr.rev.next();
    updated_entity.body.title = "Retitled Collection".into();
    assert_ne!(entity, updated_entity);

    // Outdated revision -> Conflict
    assert!(matches!(
        db.update_collection_entity_revision(&outdated_rev, DateTime::now_utc(), &updated_entity,),
        Err(RepoError::Conflict),
    ));
    // Unchanged
    assert_eq!(entity, db.load_collection_entity(id)?.1);

    // Current revision -> Success
    let current_rev = db.load_collection_entity(id)?.1.hdr.rev;
    db.update_collection_entity_revision(&current_rev, DateTime::now_local(), &updated_entity)?;
    assert_eq!(updated_entity, db.load_collection_entity(id)?.1);

    // Revert update
    db.update_collection_entity(id, DateTime::now_utc(), &entity)?;
    assert_eq!(entity, db.load_collection_entity(id)?.1);

    Ok(())
}

#[test]
fn delete_collection() -> TestResult<()> {
    let fixture = Fixture::new()?;
    let db = crate::Connection::new(&fixture.db);

    let entity = create_collection(
        &db,
        Collection {
            title: "Test Collection".into(),
            notes: None,
            kind: None,
            color: None,
            media_source_config: MediaSourceConfig {
                path_kind: media::SourcePathKind::VirtualFilePath,
                base_url: None,
            },
        },
    )
    .unwrap();
    println!("Created entity: {:?}", entity);
    let id = db.resolve_collection_id(&entity.hdr.uid)?;
    db.delete_collection_entity(id)?;
    println!("Removed entity: {}", entity.hdr.uid);
    Ok(())
}
