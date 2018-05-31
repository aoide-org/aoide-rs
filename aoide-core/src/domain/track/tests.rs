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
use mime_guess;
use serde_json;

#[test]
fn serialize_json() {
    let classifications = vec![
        Classification::new(ClassificationSubject::Energy, 0.1),
        Classification::new(ClassificationSubject::Popularity, 0.9),
    ];
    let music = MusicMetadata {
        classifications,
        loudness: Some(Loudness::EBUR128LUFS(LUFS { db: -2.3 })),
        ..Default::default()
    };
    let comments = vec![
        Comment::new_anonymous("Some anonymous notes about this track"),
    ];
    let uri = "subfolder/test.mp3";
    let source = TrackSource {
        uri: uri.to_string(),
        synchronization: Some(TrackSynchronization {
            when: Utc::now(),
            revision: EntityRevision::initial(),
        }),
        media_type: mime_guess::guess_mime_type(uri).to_string(),
        audio_content: None,
    };
    let resources = vec![
        TrackResource {
            collection: TrackCollection {
                uid: EntityUidGenerator::generate_uid(),
                since: Utc::now(),
            },
            source,
            color: Some(TrackColor::RED),
            play_counter: None,
        },
    ];
    let tags = vec![
        Tag::new_faceted(TrackTag::FACET_STYLE, "1980s", 0.8),
        Tag::new_faceted("STYLE", "1990s", 0.3),
        Tag::new_faceted(TrackTag::FACET_SESSION, "Filler", 0.6),
        Tag::new("non-faceted tag", 1.0),
    ];
    let body = TrackBody {
        resources,
        music: Some(music),
        tags,
        comments,
        ..Default::default()
    };
    let uid = EntityUidGenerator::generate_uid();
    let header = EntityHeader::with_uid(uid);
    let entity = TrackEntity { header, body };
    let entity_json = serde_json::to_string(&entity).unwrap();
    assert_ne!("{}", entity_json);
    println!("Track Entity (JSON): {}", entity_json);
}

#[test]
fn star_rating() {
    assert_eq!(0, Rating::new_anonymous(0.0).star_rating(5));
    assert_eq!(1, Rating::new_anonymous(0.01).star_rating(5));
    assert_eq!(1, Rating::new_anonymous(0.2).star_rating(5));
    assert_eq!(2, Rating::new_anonymous(0.21).star_rating(5));
    assert_eq!(2, Rating::new_anonymous(0.4).star_rating(5));
    assert_eq!(3, Rating::new_anonymous(0.41).star_rating(5));
    assert_eq!(3, Rating::new_anonymous(0.6).star_rating(5));
    assert_eq!(4, Rating::new_anonymous(0.61).star_rating(5));
    assert_eq!(4, Rating::new_anonymous(0.8).star_rating(5));
    assert_eq!(5, Rating::new_anonymous(0.81).star_rating(5));
    assert_eq!(5, Rating::new_anonymous(0.99).star_rating(5));
    assert_eq!(5, Rating::new_anonymous(1.0).star_rating(5));
    for max_stars in 4..10 {
        for stars in 0..max_stars {
            assert_eq!(
                stars,
                Rating::new_anonymous(Rating::rating_from_stars(stars, max_stars))
                    .star_rating(max_stars)
            );
        }
    }
}
