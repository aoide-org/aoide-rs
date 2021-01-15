use super::*;

use aoide_core::util::clock::DateTime;

#[test]
fn deserialize_artwork_default() {
    let json = serde_json::json!({}).to_string();
    let artwork: _core::Artwork = serde_json::from_str::<Artwork>(&json).unwrap().into();
    assert_eq!(_core::Artwork::default(), artwork);
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
    let now = DateTime::now_local();
    let json = serde_json::json!({
        "collectedAt": now.to_string(),
        "synchronizedAt": now.to_string(),
        "uri": "file:///home/test/file.mp3",
        "contentType": "audio/mpeg",
        "contentDigest": "aGVsbG8gaW50ZXJuZXR-Cg",
        "audio": {}
    })
    .to_string();
    let source: _core::Source = serde_json::from_str::<Source>(&json).unwrap().into();
    assert_eq!(
        _core::Source {
            collected_at: now,
            synchronized_at: Some(now),
            uri: "file:///home/test/file.mp3".to_string(),
            content_type: "audio/mpeg".to_string(),
            content_digest: Digest::from_encoded("aGVsbG8gaW50ZXJuZXR-Cg")
                .try_decode()
                .ok(),
            content_metadata_flags: Default::default(),
            content: _core::Content::Audio(Default::default()),
            artwork: Default::default(),
        },
        source
    );
}
