// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{media::content::ContentRevision, util::clock::OffsetDateTimeMs};

use super::*;

#[test]
fn deserialize_artwork_missing() {
    let json = serde_json::json!({"source": "missing"}).to_string();
    let artwork = serde_json::from_str::<Artwork>(&json)
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(_core::artwork::Artwork::Missing, artwork);
}

#[test]
fn serde_digest() {
    let encoded = "aGVsbG8gaW50ZXJuZXR-Cg";
    let digest = Digest::from_encoded(encoded);
    let decoded = digest.try_decode().expect("decoded");
    assert_eq!(digest, decoded.into());
}

#[test]
fn deserialize_audio_source() {
    let now = OffsetDateTimeMs::now_local_or_utc();
    let content_rev = ContentRevision::new(345);
    let json = serde_json::json!({
        "collectedAt": now.to_string(),
        "content": {
            "link": {
                "path": "/home/test file.mp3",
                "rev": Some(content_rev.to_value()),
            },
            "type": "audio/mpeg",
            "digest": "aGVsbG8gaW50ZXJuZXR-Cg",
            "audio": {},
        },
    })
    .to_string();
    let source: _core::Source = serde_json::from_str::<Source>(&json)
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(
        _core::Source {
            collected_at: now,
            content: _core::Content {
                link: _core::content::ContentLink {
                    path: "/home/test file.mp3".to_owned().into(),
                    rev: Some(content_rev),
                },
                r#type: "audio/mpeg".parse().unwrap(),
                metadata: _core::content::ContentMetadata::Audio(Default::default()),
                metadata_flags: Default::default(),
                digest: Digest::from_encoded("aGVsbG8gaW50ZXJuZXR-Cg")
                    .try_decode()
                    .ok(),
            },
            artwork: None,
        },
        source
    );
}
