// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

///////////////////////////////////////////////////////////////////////

embed_migrations!("resources/migrations/sqlite");

fn establish_connection() -> SqliteConnection {
    let connection =
        SqliteConnection::establish(":memory:").expect("in-memory database connection");
    embedded_migrations::run(&connection).expect("database schema migration");
    connection
}

#[test]
fn create_entity() {
    let connection = establish_connection();
    let repository = CollectionRepository::new(&connection);
    let entity = repository
        .create_entity(Collection {
            name: "Test Collection".into(),
            description: Some("Description".into()),
        })
        .unwrap();
    println!("Created entity: {:?}", entity);
    assert!(entity.header().is_valid());
}

#[test]
fn update_entity() {
    let connection = establish_connection();
    let repository = CollectionRepository::new(&connection);
    let mut entity = repository
        .create_entity(Collection {
            name: "Test Collection".into(),
            description: Some("Description".into()),
        })
        .unwrap();
    println!("Created entity: {:?}", entity);
    assert!(entity.header().is_valid());
    let prev_revision = entity.header().revision().clone();
    entity.body_mut().name = "Renamed Collection".into();
    let (prev_revision2, next_revision) = repository.update_entity(&entity).unwrap();
    println!("Updated entity: {:?}", entity);
    assert!(prev_revision == prev_revision2);
    assert!(prev_revision < next_revision.unwrap());
    assert!(entity.header().revision() == &prev_revision);
}

#[test]
fn delete_entity() {
    let connection = establish_connection();
    let repository = CollectionRepository::new(&connection);
    let entity = repository
        .create_entity(Collection {
            name: "Test Collection".into(),
            description: None,
        })
        .unwrap();
    println!("Created entity: {:?}", entity);
    assert!(entity.header().is_valid());
    assert_eq!(
        Some(()),
        repository.delete_entity(&entity.header().uid()).unwrap()
    );
    println!("Removed entity: {}", entity.header().uid());
}
