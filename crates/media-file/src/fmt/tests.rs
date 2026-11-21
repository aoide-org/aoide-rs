// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    media::{
        MediaDigest,
        artwork::{ApicType, Artwork, ArtworkImage, EmbeddedArtwork},
    },
    tag::{Label, PlainTag, Score},
};

use super::{reuse_unchanged_embedded_artwork, sort_tags_by_descending_score};

fn tag(label: &'static str, score: f64) -> PlainTag<'static> {
    PlainTag {
        label: Label::clamp_from(label),
        score: Score::new_unchecked(score),
    }
}

#[test]
fn sort_tags_by_descending_score_restores_the_order_and_is_stable() {
    let mut tags = vec![
        tag("Ambient", 0.5625),
        tag("Disco", 0.75),
        tag("House", 1.0),
        tag("Techno", 0.75),
    ];
    sort_tags_by_descending_score(&mut tags);
    let labels = tags
        .iter()
        .map(|tag| tag.label.as_ref().unwrap().as_str())
        .collect::<Vec<_>>();
    assert_eq!(vec!["House", "Disco", "Techno", "Ambient"], labels);
}

const IMAGE_DATA: &[u8] = b"not a real image";

fn embedded_artwork(apic_type: ApicType, media_type: &str, image_data: &[u8]) -> Artwork {
    let digest = MediaDigest::new()
        .digest_content_data(image_data)
        .finalize_reset();
    Artwork::Embedded(EmbeddedArtwork {
        image: ArtworkImage {
            apic_type,
            media_type: media_type.parse().unwrap(),
            data_size: image_data.len() as u64,
            digest,
            image_size: None,
            color: None,
            thumbnail: None,
        },
    })
}

fn reuse(old_artwork: Option<&Artwork>, mime_type: &str, image_data: &[u8]) -> Option<Artwork> {
    reuse_unchanged_embedded_artwork(
        old_artwork,
        ApicType::CoverFront,
        mime_type,
        image_data,
        &mut MediaDigest::new(),
    )
}

#[test]
fn reuse_unchanged_embedded_artwork_if_all_properties_match() {
    let old_artwork = embedded_artwork(ApicType::CoverFront, "image/jpeg", IMAGE_DATA);
    assert_eq!(
        Some(&old_artwork),
        reuse(Some(&old_artwork), "image/jpeg", IMAGE_DATA).as_ref()
    );
}

#[test]
fn do_not_reuse_embedded_artwork_if_any_property_differs() {
    let old_artwork = embedded_artwork(ApicType::CoverFront, "image/jpeg", IMAGE_DATA);
    assert!(reuse(None, "image/jpeg", IMAGE_DATA).is_none());
    assert!(reuse(Some(&Artwork::Missing), "image/jpeg", IMAGE_DATA).is_none());
    // Different image data of the same size
    assert!(reuse(Some(&old_artwork), "image/jpeg", b"not a fake image").is_none());
    // Different media type
    assert!(reuse(Some(&old_artwork), "image/png", IMAGE_DATA).is_none());
    // Unparsable media type
    assert!(reuse(Some(&old_artwork), "invalid", IMAGE_DATA).is_none());
    // Different APIC type
    let other_apic_type = embedded_artwork(ApicType::Media, "image/jpeg", IMAGE_DATA);
    assert!(reuse(Some(&other_apic_type), "image/jpeg", IMAGE_DATA).is_none());
}

#[test]
fn do_not_reuse_embedded_artwork_without_digest() {
    let old_artwork = embedded_artwork(ApicType::CoverFront, "image/jpeg", IMAGE_DATA);
    assert!(
        reuse_unchanged_embedded_artwork(
            Some(&old_artwork),
            ApicType::CoverFront,
            "image/jpeg",
            IMAGE_DATA,
            &mut MediaDigest::void(),
        )
        .is_none()
    );
}

#[test]
fn accepted_content_types_include_guessed_content_types() {
    use lofty::file::FileType;

    use super::accepted_content_types;
    use crate::util::guess_mime_from_file_path;

    for (extension, file_type) in [
        ("aif", FileType::Aiff),
        ("aiff", FileType::Aiff),
        ("flac", FileType::Flac),
        ("m4a", FileType::Mp4),
        ("mp4", FileType::Mp4),
        ("mp3", FileType::Mpeg),
        ("ogg", FileType::Vorbis),
        ("opus", FileType::Opus),
    ] {
        let content_type = guess_mime_from_file_path(format!("file.{extension}")).unwrap();
        assert!(
            accepted_content_types(file_type).contains(&content_type.essence_str()),
            "{extension}: {content_type}"
        );
    }
    assert!(accepted_content_types(FileType::Flac).contains(&"audio/x-flac"));
    assert!(!accepted_content_types(FileType::Flac).contains(&"audio/mpeg"));
    assert_eq!(accepted_content_types(FileType::Wav), &[] as &[&str]);
}
