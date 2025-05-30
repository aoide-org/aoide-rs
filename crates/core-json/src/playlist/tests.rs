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
    let tz = tz::db().get("Europe/Berlin").unwrap();
    let added_at = "2020-12-18T22:27:15+01:00[Europe/Berlin]"
        .parse::<Zoned>()
        .unwrap();
    let added_at1 = "2020-12-18T21:27:15Z".parse::<OffsetDateTimeMs>().unwrap();
    let added_at2 = "2020-12-18T22:27:15+01:00[Europe/Berlin]"
        .parse::<OffsetDateTimeMs>()
        .unwrap();
    let playlist = PlaylistWithEntries {
        playlist: Playlist {
            title: "Title".to_owned(),
            kind: Some("Kind".to_owned()),
            notes: None,
            color: None,
            iana_tz: tz.iana_name().map(ToOwned::to_owned),
            flags: 0,
        },
        entries: vec![
            Entry {
                added_at: Zoned::new(added_at1.to_utc().to_timestamp(), tz.clone()),
                item: Item::Track(TrackItem { uid: uid.clone() }),
                title: None,
                notes: None,
            },
            Entry {
                added_at: Zoned::new(added_at2.to_utc().to_timestamp(), tz.clone()),
                item: Item::Separator(Default::default()),
                title: None,
                notes: None,
            },
        ],
    };
    let playlist_json = serde_json::json!({
        "title": playlist.playlist.title,
        "kind": playlist.playlist.kind,
        "ianaTz": tz.iana_name().unwrap(),
        "entries": [
            {
                "track": {
                    "uid": uid.to_string()
                },
                "addedAt": added_at.to_string()
            },
            {
                "separator": {},
                "addedAt": added_at.to_string()
            },
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
