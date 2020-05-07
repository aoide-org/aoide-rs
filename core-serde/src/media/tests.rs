use super::*;

#[test]
fn deserialize_artwork_embedded_default() {
    let json = serde_json::json!({}).to_string();
    let artwork: _core::Artwork = serde_json::from_str::<Artwork>(&json).unwrap().into();
    assert_eq!(_core::Artwork::default(), artwork);
}

#[test]
fn deserialize_artwork_embedded() {
    let json = serde_json::json!({
        "res": "front",
    })
    .to_string();
    let artwork: _core::Artwork = serde_json::from_str::<Artwork>(&json).unwrap().into();
    assert_eq!(
        _core::Artwork {
            resource: _core::ArtworkResource::Embedded("front".to_string()),
            ..Default::default()
        },
        artwork
    );
}

#[test]
fn deserialize_artwork_uri() {
    let json = serde_json::json!({
        "uri": "file:///home/test/file.jpg",
    })
    .to_string();
    let artwork: _core::Artwork = serde_json::from_str::<Artwork>(&json).unwrap().into();
    assert_eq!(
        _core::Artwork {
            resource: _core::ArtworkResource::URI("file:///home/test/file.jpg".to_string()),
            ..Default::default()
        },
        artwork
    );
}

#[test]
fn should_fail_to_deserialize_artwork_with_both_resources() {
    let json = serde_json::json!({
        "res": "front",
        "uri": "file:///home/test/file.jpg",
    })
    .to_string();
    assert!(serde_json::from_str::<Artwork>(&json).is_err());
}

#[test]
fn deserialize_audio_source() {
    let json = serde_json::json!({
        "typ": "audio/mpeg",
        "uri": "file:///home/test/file.mp3",
        "aud": {},
    })
    .to_string();
    let source: _core::Source = serde_json::from_str::<Source>(&json).unwrap().into();
    assert_eq!(
        _core::Source {
            content: _core::Content::Audio(Default::default()),
            content_type: "audio/mpeg".to_string(),
            uri: "file:///home/test/file.mp3".to_string(),
            artwork: None,
        },
        source
    );
}
