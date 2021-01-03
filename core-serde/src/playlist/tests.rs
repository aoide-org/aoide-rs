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

#[test]
fn serialize_item_separator_dummy() {
    assert_eq!("{}", serde_json::to_string(&SeparatorDummy {}).unwrap());
}

#[test]
fn deserialize_playlist() {
    let uid: aoide_core::entity::EntityUid = "MAdeyPtrDVSMnwpriPA5anaD66xw5iP1s".parse().unwrap();
    let added_at1: aoide_core::util::clock::DateTime = "2020-12-18T21:27:15Z".parse().unwrap();
    let added_at2 = aoide_core::util::clock::DateTime::now_utc();
    let playlist = PlaylistWithEntries {
        playlist: Playlist {
            collected_at: added_at1.into(),
            title: "Title".to_string(),
            kind: Some("Kind".to_string()),
            notes: None,
            color: None,
        },
        entries: vec![
            Entry {
                added_at: added_at1.into(),
                item: Item::Track(track::Item {
                    uid: uid.clone().into(),
                }),
                title: None,
                notes: None,
            },
            Entry {
                added_at: added_at2.into(),
                item: Item::Separator(SeparatorDummy {}),
                title: None,
                notes: None,
            },
        ],
    };
    let playlist_json = serde_json::json!({
        "collectedAt": added_at1.to_string(),
        "title": playlist.playlist.title.clone(),
        "kind": playlist.playlist.kind.clone(),
        "entries": [
            {
                "track": {
                    "uid": uid.to_string()
                },
                "addedAt": added_at1.to_string()
            },
            {
                "separator": {},
                "addedAt": added_at2.to_string()
            }
        ]
    })
    .to_string();
    let playlist_deserialized = serde_json::from_str(&playlist_json).unwrap();
    assert_eq!(playlist, playlist_deserialized);
    // Roundtrip
    let playlist_serialized = serde_json::to_string(&playlist).unwrap();
    assert_eq!(
        playlist,
        serde_json::from_str(&playlist_serialized).unwrap()
    );
}
