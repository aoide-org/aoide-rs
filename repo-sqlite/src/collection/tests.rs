// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

embed_migrations!("migrations");

fn establish_connection() -> SqliteConnection {
    let connection =
        SqliteConnection::establish(":memory:").expect("in-memory database connection");
    embedded_migrations::run(&connection).expect("database schema migration");
    connection
}

fn create_collection(repo: &dyn Repo, collection: Collection) -> RepoResult<Entity> {
    let entity = Entity::new(EntityHeader::initial_random(), collection);
    repo.insert_collection(&entity).and(Ok(entity))
}

#[test]
fn insert_collection() {
    let db_connection = establish_connection();
    let connection = crate::Connection::from(&db_connection);
    let entity = create_collection(
        &connection,
        Collection {
            name: "Test Collection".into(),
            description: Some("Description".into()),
        },
    )
    .unwrap();
    println!("Created entity: {:?}", entity);
}

#[test]
fn update_collection() {
    let db_connection = establish_connection();
    let connection = crate::Connection::from(&db_connection);
    let mut entity = create_collection(
        &connection,
        Collection {
            name: "Test Collection".into(),
            description: Some("Description".into()),
        },
    )
    .unwrap();
    println!("Created entity: {:?}", entity);
    let prev_rev = entity.hdr.rev;
    entity.body.name = "Renamed Collection".into();
    let (prev_rev2, next_rev) = connection.update_collection(&entity).unwrap();
    println!("Updated entity: {:?}", entity);
    assert!(prev_rev == prev_rev2);
    assert!(prev_rev < next_rev.unwrap());
    assert!(entity.hdr.rev == prev_rev);
}

#[test]
fn delete_collection() {
    let db_connection = establish_connection();
    let connection = crate::Connection::from(&db_connection);
    let entity = create_collection(
        &connection,
        Collection {
            name: "Test Collection".into(),
            description: None,
        },
    )
    .unwrap();
    println!("Created entity: {:?}", entity);
    assert_eq!(
        Some(()),
        connection.delete_collection(&entity.hdr.uid).unwrap()
    );
    println!("Removed entity: {}", entity.hdr.uid);
}
