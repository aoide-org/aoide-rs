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

use aoide_core::entity::EntityHeader;
use test_env_log::test;

embed_migrations!("migrations");

fn establish_connection() -> SqliteConnection {
    let connection =
        SqliteConnection::establish(":memory:").expect("in-memory database connection");
    embedded_migrations::run(&connection).expect("database schema migration");
    connection
}

fn create_collection(repo: &dyn EntityRepo, collection: Collection) -> RepoResult<Entity> {
    let entity = Entity::new(EntityHeader::initial_random(), collection);
    repo.insert_collection_entity(DateTime::now(), &entity)
        .and(Ok(entity))
}

#[test]
fn insert_collection() {
    let db_connection = establish_connection();
    let connection = crate::Connection::from(&db_connection);
    let entity = create_collection(
        &connection,
        Collection {
            title: "Test Collection".into(),
            notes: Some("Some personal notes".into()),
            kind: None,
            color: None,
        },
    )
    .unwrap();
    println!("Created entity: {:?}", entity);
}

#[test]
fn update_collection() -> RepoResult<()> {
    let db_connection = establish_connection();
    let connection = crate::Connection::from(&db_connection);

    let mut entity = create_collection(
        &connection,
        Collection {
            title: "Test Collection".into(),
            notes: Some("Description".into()),
            kind: None,
            color: None,
        },
    )?;
    let id = connection.resolve_collection_id(&entity.hdr.uid)?;

    // Bump revision number for testing
    let outdated_rev = entity.hdr.rev;
    entity.hdr.rev = outdated_rev.next();
    connection.update_collection_entity(id, DateTime::now(), &entity)?;
    assert_eq!(entity, connection.load_collection_entity(id)?.1);

    // Prepare update
    let mut updated_entity = entity.clone();
    updated_entity.hdr.rev = updated_entity.hdr.rev.next();
    updated_entity.body.title = "Retitled Collection".into();
    assert_ne!(entity, updated_entity);

    // Outdated revision -> Conflict
    assert!(matches!(
        connection.update_collection_entity_revision(
            DateTime::now(),
            &outdated_rev,
            &updated_entity,
        ),
        Err(RepoError::Conflict),
    ));
    // Unchanged
    assert_eq!(entity, connection.load_collection_entity(id)?.1);

    // Current revision -> Success
    let current_rev = connection.load_collection_entity(id)?.1.hdr.rev;
    connection.update_collection_entity_revision(DateTime::now(), &current_rev, &updated_entity)?;
    assert_eq!(updated_entity, connection.load_collection_entity(id)?.1);

    // Revert update
    connection.update_collection_entity(id, DateTime::now(), &entity)?;
    assert_eq!(entity, connection.load_collection_entity(id)?.1);

    Ok(())
}

#[test]
fn delete_collection() -> RepoResult<()> {
    let db_connection = establish_connection();
    let connection = crate::Connection::from(&db_connection);
    let entity = create_collection(
        &connection,
        Collection {
            title: "Test Collection".into(),
            notes: None,
            kind: None,
            color: None,
        },
    )
    .unwrap();
    println!("Created entity: {:?}", entity);
    let id = connection.resolve_collection_id(&entity.hdr.uid)?;
    connection.delete_collection_entity(id)?;
    println!("Removed entity: {}", entity.hdr.uid);
    Ok(())
}
