// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::util::clock::OffsetDateTimeMs;

use super::*;

#[test]
fn serialize_item_default_separator() {
    assert_eq!(
        "{}",
        serde_json::to_string(&SeparatorItem::default()).unwrap()
    );
}

#[test]
fn deserialize_playlist() {
    let uid: EntityUid = "01AN4Z07BY79KA1307SR9X4MV3".parse().unwrap();
    let added_at1 = "2020-12-18T21:27:15Z".parse::<OffsetDateTimeMs>().unwrap();
    let added_at2 = "2020-12-18T21:27:15-01:00"
        .parse::<OffsetDateTimeMs>()
        .unwrap();
    let added_at3 = OffsetDateTimeMs::now_utc();
    let playlist = PlaylistWithEntries {
        playlist: Playlist {
            title: "Title".to_string(),
            kind: Some("Kind".to_string()),
            notes: None,
            color: None,
            flags: 0,
        },
        entries: vec![
            Entry {
                added_at: added_at1.clone().into(),
                item: Item::Track(TrackItem { uid: uid.clone() }),
                title: None,
                notes: None,
            },
            Entry {
                added_at: added_at2.clone().into(),
                item: Item::Separator(Default::default()),
                title: None,
                notes: None,
            },
            Entry {
                added_at: added_at3.clone().into(),
                item: Item::Separator(SeparatorItem {
                    kind: Some("Kind".into()),
                }),
                title: None,
                notes: None,
            },
        ],
    };
    let playlist_json = serde_json::json!({
        "title": playlist.playlist.title,
        "kind": playlist.playlist.kind,
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
            },
            {
                "separator": {
                    "kind": "Kind"
                },
                "addedAt": added_at3.to_string()
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
