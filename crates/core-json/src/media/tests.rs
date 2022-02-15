use super::*;

use aoide_core::{media::content::ContentRevision, util::clock::DateTime};

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
    let now = DateTime::now_local_or_utc();
    let content_rev = ContentRevision::new(345);
    let json = serde_json::json!({
        "collectedAt": now.to_string(),
        "contentLink": {
            "path": "/home/test file.mp3",
            "rev": Some(content_rev.to_value()),
        },
        "contentType": "audio/mpeg",
        "contentDigest": "aGVsbG8gaW50ZXJuZXR-Cg",
        "audio": {},
        "advisoryRating": 0,
    })
    .to_string();
    let source: _core::Source = serde_json::from_str::<Source>(&json)
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(
        _core::Source {
            collected_at: now,
            content_link: _core::content::ContentLink {
                path: _core::content::ContentPath::new("/home/test file.mp3".to_owned()),
                rev: Some(content_rev),
            },
            content_type: "audio/mpeg".parse().unwrap(),
            content_metadata: _core::content::ContentMetadata::Audio(Default::default()),
            content_metadata_flags: Default::default(),
            content_digest: Digest::from_encoded("aGVsbG8gaW50ZXJuZXR-Cg")
                .try_decode()
                .ok(),
            advisory_rating: Some(_core::AdvisoryRating::Unrated),
            artwork: None,
        },
        source
    );
}
