// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

embed_migrations!("db/migrations/sqlite");

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
        .create_entity(CollectionBody {
            name: "Test Collection".into(),
            description: Some("Description".into()),
        })
        .unwrap();
    println!("Created entity: {:?}", entity);
    assert!(entity.is_valid());
}

#[test]
fn update_entity() {
    let connection = establish_connection();
    let repository = CollectionRepository::new(&connection);
    let mut entity = repository
        .create_entity(CollectionBody {
            name: "Test Collection".into(),
            description: Some("Description".into()),
        })
        .unwrap();
    println!("Created entity: {:?}", entity);
    assert!(entity.is_valid());
    let prev_revision = entity.header().revision();
    entity.body_mut().name = "Renamed Collection".into();
    let (prev_revision2, next_revision) = repository.update_entity(&entity).unwrap().unwrap();
    println!("Updated entity: {:?}", entity);
    assert!(prev_revision == prev_revision2);
    assert!(prev_revision < next_revision);
    assert!(entity.header().revision() == prev_revision);
    entity.update_revision(next_revision);
    assert!(entity.header().revision() == next_revision);
}

#[test]
fn remove_entity() {
    let connection = establish_connection();
    let repository = CollectionRepository::new(&connection);
    let entity = repository
        .create_entity(CollectionBody {
            name: "Test Collection".into(),
            description: None,
        })
        .unwrap();
    println!("Created entity: {:?}", entity);
    assert!(entity.is_valid());
    assert_eq!(
        Some(()),
        repository.remove_entity(&entity.header().uid()).unwrap()
    );
    println!("Removed entity: {}", entity.header().uid());
}
