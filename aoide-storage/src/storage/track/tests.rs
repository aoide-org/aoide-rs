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

use chrono::Utc;

use api::collection::Collections;

use storage::collection::CollectionRepository;

use aoide_core::domain::collection::Collection;

embed_migrations!("resources/migrations/sqlite");

fn establish_connection() -> SqliteConnection {
    let connection =
        SqliteConnection::establish(":memory:").expect("in-memory database connection");
    embedded_migrations::run(&connection).expect("database schema migration");
    connection
}

#[test]
fn search_distinct_with_multiple_sources() {
    let connection = establish_connection();
    let collection_repo = CollectionRepository::new(&connection);
    let collection1 = collection_repo
        .create_entity(Collection {
            name: "Collection 1".into(),
            description: None,
        })
        .unwrap();
    let collection2 = collection_repo
        .create_entity(Collection {
            name: "Collection 2".into(),
            description: None,
        })
        .unwrap();
    let track_repo = TrackRepository::new(&connection);
    let track_res1 = TrackResource {
        collection: TrackCollection {
            uid: *collection1.header().uid(),
            since: Utc::now(),
        },
        source: TrackSource {
            uri: "testfile1.mp3".into(),
            media_type: "audio/mpeg".into(),
            ..Default::default()
        },
        color: None,
        play_count: None,
    };
    let track_res2 = TrackResource {
        collection: TrackCollection {
            uid: *collection2.header().uid(),
            since: Utc::now(),
        },
        source: TrackSource {
            uri: "testfile2.flac".into(),
            media_type: "audio/flac".into(),
            ..Default::default()
        },
        color: None,
        play_count: None,
    };
    let _track = track_repo
        .create_entity(
            Track {
                resources: vec![track_res1, track_res2],
                ..Default::default()
            },
            SerializationFormat::JSON,
        )
        .unwrap();
    let search_all_count = track_repo
        .search_entities(
            None,
            Default::default(),
            SearchTracksParams {
                ..Default::default()
            },
        )
        .unwrap()
        .len();
    assert_eq!(1, search_all_count);
    let search_testfile_count = track_repo
        .search_entities(
            None,
            Default::default(),
            SearchTracksParams {
                phrase_filter: Some(PhraseFilter {
                    modifier: None,
                    fields: vec![PhraseField::MediaSource],
                    phrase: "testfile".into(),
                }),
                ..Default::default()
            },
        )
        .unwrap()
        .len();
    assert_eq!(1, search_testfile_count);
    let search_testfile1_count = track_repo
        .search_entities(
            None,
            Default::default(),
            SearchTracksParams {
                phrase_filter: Some(PhraseFilter {
                    modifier: None,
                    fields: vec![PhraseField::MediaSource],
                    phrase: "testfile1".into(),
                }),
                ..Default::default()
            },
        )
        .unwrap()
        .len();
    assert_eq!(1, search_testfile1_count);
    let search_testfile2_count = track_repo
        .search_entities(
            None,
            Default::default(),
            SearchTracksParams {
                phrase_filter: Some(PhraseFilter {
                    modifier: None,
                    fields: vec![PhraseField::MediaSource],
                    phrase: "testfile2".into(),
                }),
                ..Default::default()
            },
        )
        .unwrap()
        .len();
    assert_eq!(1, search_testfile2_count);
    let search_testfile3_count = track_repo
        .search_entities(
            None,
            Default::default(),
            SearchTracksParams {
                phrase_filter: Some(PhraseFilter {
                    modifier: None,
                    fields: vec![PhraseField::MediaSource],
                    phrase: "testfile3".into(),
                }),
                ..Default::default()
            },
        )
        .unwrap()
        .len();
    assert_eq!(0, search_testfile3_count);
}
