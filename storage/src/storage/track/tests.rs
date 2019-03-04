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
fn search_distinct_with_multiple_sources() {
    let connection = establish_connection();
    let repository = TrackRepository::new(&connection);
    let track_src1 = TrackSource {
        content_uri: "testfile1.mp3".into(),
        content_type: "audio/mpeg".into(),
        ..Default::default()
    };
    let track_src2 = TrackSource {
        content_uri: "testfile2.flac".into(),
        content_type: "audio/flac".into(),
        ..Default::default()
    };
    let _track = repository
        .create_entity(
            Track {
                sources: vec![track_src1, track_src2],
                ..Default::default()
            },
            SerializationFormat::JSON,
        )
        .unwrap();
    let search_all_count = repository
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
    let search_testfile_count = repository
        .search_entities(
            None,
            Default::default(),
            SearchTracksParams {
                phrase_filter: Some(PhraseFilter {
                    modifier: None,
                    fields: vec![PhraseField::SourceUri],
                    phrase: "testfile".into(),
                }),
                ..Default::default()
            },
        )
        .unwrap()
        .len();
    assert_eq!(1, search_testfile_count);
    let search_testfile1_count = repository
        .search_entities(
            None,
            Default::default(),
            SearchTracksParams {
                phrase_filter: Some(PhraseFilter {
                    modifier: None,
                    fields: vec![PhraseField::SourceUri],
                    phrase: "testfile1".into(),
                }),
                ..Default::default()
            },
        )
        .unwrap()
        .len();
    assert_eq!(1, search_testfile1_count);
    let search_testfile2_count = repository
        .search_entities(
            None,
            Default::default(),
            SearchTracksParams {
                phrase_filter: Some(PhraseFilter {
                    modifier: None,
                    fields: vec![PhraseField::SourceUri],
                    phrase: "testfile2".into(),
                }),
                ..Default::default()
            },
        )
        .unwrap()
        .len();
    assert_eq!(1, search_testfile2_count);
    let search_testfile3_count = repository
        .search_entities(
            None,
            Default::default(),
            SearchTracksParams {
                phrase_filter: Some(PhraseFilter {
                    modifier: None,
                    fields: vec![PhraseField::SourceUri],
                    phrase: "testfile3".into(),
                }),
                ..Default::default()
            },
        )
        .unwrap()
        .len();
    assert_eq!(0, search_testfile3_count);
}
