// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Roundtrip tests for importing and exporting file tags.
//!
//! Each test starts with an empty media file from `tests/assets`, copies it into
//! a temporary file, and then repeatedly modifies the imported track metadata,
//! exports it into the file, and re-imports it. No metadata must get lost.

use std::{fs::File, io::BufReader, path::Path};

use aoide_core::{
    PlainTag, TagFacetId, TagLabel, Tags, TagsMap, Track,
    audio::signal::LoudnessLufs,
    media::content::{AudioContentMetadata, ContentLink, ContentMetadata},
    music::{
        key::{KeyCode, KeySignature},
        tempo::TempoBpm,
    },
    track::{
        AdvisoryRating,
        actor::{Actor, Kind as ActorKind, Role as ActorRole},
        album::{Album, Kind as AlbumKind},
        index::{Index, Indexes},
        tag::{
            FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING,
            FACET_ID_ISRC, FACET_ID_MBID_RECORDING, FACET_ID_MBID_RELEASE,
            FACET_ID_MBID_RELEASE_GROUP, FACET_ID_MBID_TRACK, FACET_ID_MOOD, FACET_ID_XID,
        },
        title::{Kind as TitleKind, Title},
    },
    util::clock::{DateOrDateTime, OffsetDateTimeMs, YyyyMmDdDate},
};
use aoide_media_file::io::{
    export::{ExportTrackConfig, export_track_to_file_with_type},
    import::{ImportTrack, ImportTrackConfig, Reader, import_into_track},
};
use lofty::file::FileType;
use mime::Mime;
use nonicle::{Canonical, CanonicalizeInto as _};
use tempfile::NamedTempFile;

#[derive(Debug, Clone, Copy)]
struct TestFile {
    asset_path: &'static str,
    suffix: &'static str,
    content_type: &'static str,
    file_type: FileType,
    // Metadata that could not be stored in the file tag (limitations of lofty).
    unsupported_actors: &'static [(ActorRole, ActorKind)],
    unsupported_facets: &'static [&'static TagFacetId],
    unsupported_advisory_rating: bool,
}

const MP3: TestFile = TestFile {
    asset_path: "tests/assets/empty.mp3",
    suffix: ".mp3",
    content_type: "audio/mpeg",
    file_type: FileType::Mpeg,
    unsupported_actors: &[
        // No mapping for TMCL.
        (ActorRole::Performer, ActorKind::Summary),
        // Shares the TEXT frame with ActorRole::Lyricist.
        (ActorRole::Writer, ActorKind::Summary),
    ],
    unsupported_facets: &[FACET_ID_DESCRIPTION, FACET_ID_XID],
    unsupported_advisory_rating: false,
};

const M4A: TestFile = TestFile {
    asset_path: "tests/assets/empty.m4a",
    suffix: ".m4a",
    content_type: "audio/m4a",
    file_type: FileType::Mp4,
    unsupported_actors: &[
        (ActorRole::Arranger, ActorKind::Summary),
        (ActorRole::Performer, ActorKind::Summary),
        (ActorRole::Writer, ActorKind::Summary),
    ],
    unsupported_facets: &[],
    unsupported_advisory_rating: false,
};

const FLAC: TestFile = TestFile {
    asset_path: "tests/assets/empty.flac",
    suffix: ".flac",
    content_type: "audio/flac",
    file_type: FileType::Flac,
    unsupported_actors: &[(ActorRole::Composer, ActorKind::Sorting)],
    unsupported_facets: &[FACET_ID_XID],
    unsupported_advisory_rating: true,
};

impl TestFile {
    fn content_type(&self) -> Mime {
        self.content_type.parse().unwrap()
    }

    fn copy_to_temp_file(&self) -> NamedTempFile {
        let temp_file = tempfile::Builder::new()
            .suffix(self.suffix)
            .tempfile()
            .unwrap();
        std::fs::copy(self.asset_path, temp_file.path()).unwrap();
        temp_file
    }

    /// Remove all metadata from the track that is not supported by the file format.
    fn remove_unsupported_metadata(&self, track: &mut Track) {
        let mut actors = std::mem::take(&mut track.actors).untie();
        actors.retain(|actor| !self.unsupported_actors.contains(&(actor.role, actor.kind)));
        track.actors = actors.canonicalize_into();
        let mut tags_map = TagsMap::from(std::mem::take(&mut track.tags).untie());
        for facet_id in self.unsupported_facets {
            tags_map.remove(facet_id);
        }
        track.tags = tags_map.canonicalize_into();
        if self.unsupported_advisory_rating {
            track.advisory_rating = None;
        }
    }
}

fn import_track(
    test_file: &TestFile,
    path: &Path,
    config: &ImportTrackConfig,
    import_track: ImportTrack,
) -> Track {
    let content_link = ContentLink {
        path: Default::default(),
        rev: None,
    };
    let mut track = import_track.with_content(content_link, test_file.content_type());
    let file = Box::new(File::open(path).unwrap());
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let issues = import_into_track(&mut reader, config, &mut track).unwrap();
    assert!(issues.is_empty(), "import issues: {issues:?}");
    track
}

fn import_new_track(test_file: &TestFile, path: &Path, config: &ImportTrackConfig) -> Track {
    import_track(
        test_file,
        path,
        config,
        ImportTrack::NewTrack {
            collected_at: OffsetDateTimeMs::now_utc(),
        },
    )
}

fn export_track(
    test_file: &TestFile,
    temp_file: &mut NamedTempFile,
    config: &ExportTrackConfig,
    track: &mut Track,
) {
    export_track_to_file_with_type(
        temp_file.as_file_mut(),
        test_file.file_type,
        config,
        track,
        None,
    )
    .unwrap();
}

/// Export the track, re-import it, and verify that nothing got lost.
///
/// Returns the re-imported track.
fn export_and_reimport(
    test_file: &TestFile,
    temp_file: &mut NamedTempFile,
    import_config: &ImportTrackConfig,
    export_config: &ExportTrackConfig,
    mut track: Track,
) -> Track {
    export_track(test_file, temp_file, export_config, &mut track);

    // Re-import into a new track from scratch.
    let mut reimported = import_new_track(test_file, temp_file.path(), import_config);
    reimported.media_source.collected_at = track.media_source.collected_at;
    // The audio properties and the artwork of files without any file tags
    // are not imported. Take them from the re-imported track that is
    // guaranteed to have file tags after the export.
    {
        let ContentMetadata::Audio(audio) = &mut track.media_source.content.metadata;
        let ContentMetadata::Audio(reimported_audio) = &reimported.media_source.content.metadata;
        *audio = AudioContentMetadata {
            loudness: audio.loudness,
            ..reimported_audio.clone()
        };
        track
            .media_source
            .artwork
            .clone_from(&reimported.media_source.artwork);
    }
    assert_eq!(track, reimported);

    // Re-import into the existing track, i.e. update it.
    let updated = import_track(
        test_file,
        temp_file.path(),
        import_config,
        ImportTrack::UpdateTrack(track.clone()),
    );
    assert_eq!(track, updated);

    reimported
}

fn label(label: &str) -> TagLabel<'static> {
    TagLabel::new(label.to_owned().into())
}

fn plain_tag(label_str: &str) -> PlainTag<'static> {
    PlainTag {
        label: Some(label(label_str)),
        ..Default::default()
    }
}

fn title(kind: TitleKind, name: &str) -> Title {
    Title {
        kind,
        name: name.to_owned(),
    }
}

fn actor(role: ActorRole, kind: ActorKind, name: &str) -> Actor {
    Actor {
        role,
        kind,
        name: name.to_owned(),
        role_notes: None,
    }
}

fn date(year: u16, month: u8, day: u8) -> DateOrDateTime {
    let yyyymmdd = i32::from(year) * 10_000 + i32::from(month) * 100 + i32::from(day);
    DateOrDateTime::Date(YyyyMmDdDate::new_unchecked(yyyymmdd))
}

fn set_loudness(track: &mut Track, loudness: Option<LoudnessLufs>) {
    let ContentMetadata::Audio(audio) = &mut track.media_source.content.metadata;
    audio.loudness = loudness;
}

/// Tags that are stored in their own, dedicated file tag.
fn file_tags_map() -> TagsMap<'static> {
    let mut tags_map = TagsMap::default();
    // Multi-valued file tags.
    tags_map.insert(
        FACET_ID_GENRE,
        vec![plain_tag("Deep House"), plain_tag("Tech House")],
    );
    tags_map.insert(FACET_ID_MOOD, vec![plain_tag("Happy"), plain_tag("Dreamy")]);
    // Single-valued file tags.
    tags_map.insert_one(FACET_ID_COMMENT, plain_tag("A comment"));
    tags_map.insert_one(FACET_ID_DESCRIPTION, plain_tag("A description"));
    tags_map.insert_one(FACET_ID_GROUPING, plain_tag("A grouping"));
    tags_map.insert_one(FACET_ID_ISRC, plain_tag("USRC17607839"));
    tags_map.insert_one(FACET_ID_XID, plain_tag("xid:1234567890"));
    tags_map.insert_one(
        FACET_ID_MBID_RECORDING,
        plain_tag("e6a5f1a4-8ba1-4a8c-9a51-6c4b30d1e7d1"),
    );
    tags_map.insert_one(
        FACET_ID_MBID_TRACK,
        plain_tag("5b0c4a2e-1e6f-4e1d-9d4a-2e7c3f0a9b11"),
    );
    tags_map.insert_one(
        FACET_ID_MBID_RELEASE,
        plain_tag("0a6b7c3d-5e4f-4a1b-8c2d-9e0f1a2b3c4d"),
    );
    tags_map.insert_one(
        FACET_ID_MBID_RELEASE_GROUP,
        plain_tag("7f8e9d0c-1b2a-4c3d-8e5f-6a7b8c9d0e1f"),
    );
    tags_map
}

fn populate_track_metadata(track: &mut Track, tags: Canonical<Tags<'static>>) {
    set_loudness(track, Some(LoudnessLufs::new(-9.5)));

    track.metrics.tempo_bpm = Some(TempoBpm::new(123.456));
    track.metrics.key_signature = Some(KeySignature::new(KeyCode::Amin));

    track.titles = vec![
        title(TitleKind::Main, "Track Title"),
        title(TitleKind::Sorting, "Title, Track"),
        title(TitleKind::Sub, "Track Subtitle"),
        title(TitleKind::Work, "Work Title"),
        title(TitleKind::Movement, "Movement Title"),
    ]
    .canonicalize_into();

    track.actors = vec![
        actor(ActorRole::Artist, ActorKind::Summary, "Artist 1 & Artist 2"),
        actor(ActorRole::Artist, ActorKind::Individual, "Artist 1"),
        actor(ActorRole::Artist, ActorKind::Individual, "Artist 2"),
        actor(
            ActorRole::Artist,
            ActorKind::Sorting,
            "1, Artist & 2, Artist",
        ),
        actor(ActorRole::Arranger, ActorKind::Summary, "Arranger"),
        actor(ActorRole::Composer, ActorKind::Summary, "Composer"),
        actor(ActorRole::Composer, ActorKind::Sorting, "Composer, The"),
        actor(ActorRole::Conductor, ActorKind::Summary, "Conductor"),
        actor(ActorRole::Director, ActorKind::Summary, "Director"),
        actor(ActorRole::Engineer, ActorKind::Summary, "Engineer"),
        actor(ActorRole::Lyricist, ActorKind::Summary, "Lyricist"),
        actor(ActorRole::MixDj, ActorKind::Summary, "Mix DJ"),
        actor(ActorRole::MixEngineer, ActorKind::Summary, "Mix Engineer"),
        actor(ActorRole::Performer, ActorKind::Summary, "Performer"),
        actor(ActorRole::Producer, ActorKind::Summary, "Producer"),
        actor(ActorRole::Remixer, ActorKind::Summary, "Remixer"),
        actor(ActorRole::Writer, ActorKind::Summary, "Writer"),
    ]
    .canonicalize_into();

    track.album = Canonical::tie(Album {
        kind: Some(AlbumKind::Compilation),
        titles: vec![
            title(TitleKind::Main, "Album Title"),
            title(TitleKind::Sorting, "Title, Album"),
            title(TitleKind::Sub, "Album Subtitle"),
        ]
        .canonicalize_into(),
        actors: vec![
            actor(ActorRole::Artist, ActorKind::Summary, "Various Artists"),
            actor(ActorRole::Artist, ActorKind::Sorting, "Artists, Various"),
        ]
        .canonicalize_into(),
    });

    track.indexes = Indexes {
        track: Index {
            number: Some(3),
            total: Some(12),
        },
        disc: Index {
            number: Some(1),
            total: Some(2),
        },
        movement: Index {
            number: Some(2),
            total: Some(4),
        },
    };

    track.recorded_at = Some(date(2023, 11, 24));
    track.released_at = Some(date(2024, 3, 15));
    track.released_orig_at = Some(date(1999, 1, 1));

    track.publisher = Some("Publisher".to_owned());
    track.copyright = Some("(C) Copyright".to_owned());
    track.advisory_rating = Some(AdvisoryRating::Explicit);

    track.tags = tags;
}

/// Modify, add, and remove metadata.
fn modify_track_metadata(track: &mut Track, tags: Canonical<Tags<'static>>) {
    set_loudness(track, Some(LoudnessLufs::new(-12.25)));

    // Change from fractional to integer BPM
    track.metrics.tempo_bpm = Some(TempoBpm::new(128.0));
    track.metrics.key_signature = Some(KeySignature::new(KeyCode::Fmaj));

    track.titles = vec![
        title(TitleKind::Main, "Modified Track Title"),
        title(TitleKind::Sub, "Modified Track Subtitle"),
        title(TitleKind::Work, "Modified Work Title"),
        title(TitleKind::Movement, "Modified Movement Title"),
    ]
    .canonicalize_into();

    track.actors = vec![
        actor(ActorRole::Artist, ActorKind::Summary, "Modified Artist"),
        actor(ActorRole::Composer, ActorKind::Summary, "Modified Composer"),
        actor(ActorRole::Remixer, ActorKind::Summary, "Modified Remixer"),
    ]
    .canonicalize_into();

    track.album = Canonical::tie(Album {
        kind: Some(AlbumKind::NoCompilation),
        titles: vec![
            title(TitleKind::Main, "Modified Album Title"),
            title(TitleKind::Sub, "Modified Album Subtitle"),
        ]
        .canonicalize_into(),
        actors: vec![actor(
            ActorRole::Artist,
            ActorKind::Summary,
            "Modified Album Artist",
        )]
        .canonicalize_into(),
    });

    track.indexes = Indexes {
        track: Index {
            number: Some(4),
            total: Some(13),
        },
        disc: Default::default(),
        movement: Index {
            number: Some(3),
            total: Some(4),
        },
    };

    track.recorded_at = Some(date(2024, 0, 0));
    track.released_at = None;
    track.released_orig_at = Some(date(2000, 2, 0));

    track.publisher = None;
    track.copyright = Some("(C) Modified Copyright".to_owned());
    track.advisory_rating = Some(AdvisoryRating::Clean);

    track.tags = tags;
}

fn modified_file_tags_map() -> TagsMap<'static> {
    let mut tags_map = TagsMap::default();
    tags_map.insert(
        FACET_ID_GENRE,
        vec![plain_tag("Techno"), plain_tag("Deep House")],
    );
    tags_map.insert_one(FACET_ID_COMMENT, plain_tag("A modified comment"));
    tags_map.insert_one(FACET_ID_GROUPING, plain_tag("A modified grouping"));
    tags_map.insert_one(FACET_ID_ISRC, plain_tag("USRC17607840"));
    tags_map
}

fn clear_track_metadata(track: &mut Track) {
    set_loudness(track, None);
    track.metrics = Default::default();
    track.titles = Default::default();
    track.actors = Default::default();
    track.album = Default::default();
    track.indexes = Default::default();
    track.recorded_at = None;
    track.released_at = None;
    track.released_orig_at = None;
    track.publisher = None;
    track.copyright = None;
    track.advisory_rating = None;
    track.tags = Default::default();
}

fn roundtrip(
    test_file: &TestFile,
    import_config: &ImportTrackConfig,
    export_config: &ExportTrackConfig,
    tags: Canonical<Tags<'static>>,
    modified_tags: Canonical<Tags<'static>>,
) {
    let mut temp_file = test_file.copy_to_temp_file();

    // Import the empty file.
    let empty_track = import_new_track(test_file, temp_file.path(), import_config);
    assert!(empty_track.titles.is_empty());
    assert!(empty_track.actors.is_empty());
    assert!(empty_track.tags.is_empty());

    // Populate all metadata.
    let mut track = empty_track.clone();
    populate_track_metadata(&mut track, tags);
    test_file.remove_unsupported_metadata(&mut track);
    let track = export_and_reimport(
        test_file,
        &mut temp_file,
        import_config,
        export_config,
        track,
    );

    // Exporting the same track again must not change anything.
    let track = export_and_reimport(
        test_file,
        &mut temp_file,
        import_config,
        export_config,
        track,
    );

    // Modify, add, and remove metadata.
    let mut track = track;
    modify_track_metadata(&mut track, modified_tags);
    test_file.remove_unsupported_metadata(&mut track);
    let mut track = export_and_reimport(
        test_file,
        &mut temp_file,
        import_config,
        export_config,
        track,
    );

    // Remove all metadata.
    clear_track_metadata(&mut track);
    let track = export_and_reimport(
        test_file,
        &mut temp_file,
        import_config,
        export_config,
        track,
    );
    assert!(track.titles.is_empty());
    assert!(track.actors.is_empty());
    assert!(track.tags.is_empty());
}

fn roundtrip_file_tags(test_file: &TestFile) {
    let import_config = ImportTrackConfig::default();
    let export_config = ExportTrackConfig::default();
    roundtrip(
        test_file,
        &import_config,
        &export_config,
        file_tags_map().canonicalize_into(),
        modified_file_tags_map().canonicalize_into(),
    );
}

#[test]
fn roundtrip_file_tags_mp3() {
    roundtrip_file_tags(&MP3);
}

#[test]
fn roundtrip_file_tags_m4a() {
    roundtrip_file_tags(&M4A);
}

#[test]
fn roundtrip_file_tags_flac() {
    roundtrip_file_tags(&FLAC);
}

#[cfg(feature = "gigtag")]
mod gigtag {
    use aoide_core::TagScore;
    use aoide_media_file::io::import::ImportTrackFlags;

    use super::*;

    fn plain_tag_with_score(label_str: &str, score_value: f64) -> PlainTag<'static> {
        PlainTag {
            label: Some(label(label_str)),
            score: TagScore::clamp_from(score_value),
        }
    }

    fn facet_id(facet: &str) -> TagFacetId {
        TagFacetId::clamp_from(facet).unwrap()
    }

    /// Tags that could not be stored in a dedicated file tag and
    /// need to be encoded as gig tags.
    fn insert_gig_tags(tags_map: &mut TagsMap<'static>) {
        // Plain tags without a facet.
        tags_map.insert(
            aoide_core::TagFacetKey::unfaceted(),
            vec![plain_tag("Plain Tag"), plain_tag_with_score("Scored", 0.75)],
        );
        // Custom facets with and without labels and scores.
        tags_map.insert(
            facet_id("vibe"),
            vec![plain_tag("Warm"), plain_tag_with_score("Uplifting", 0.5)],
        );
        tags_map.insert_one(
            facet_id("energy"),
            PlainTag {
                label: None,
                score: TagScore::clamp_from(0.8),
            },
        );
        // Date-like facet without a label and default score.
        tags_map.insert_one(
            facet_id("played@20240315"),
            PlainTag {
                label: None,
                ..Default::default()
            },
        );
    }

    fn gig_tags_map() -> TagsMap<'static> {
        let mut tags_map = file_tags_map();
        insert_gig_tags(&mut tags_map);
        tags_map
    }

    fn modified_gig_tags_map() -> TagsMap<'static> {
        let mut tags_map = modified_file_tags_map();
        tags_map.insert(
            aoide_core::TagFacetKey::unfaceted(),
            vec![plain_tag_with_score("Scored", 0.25)],
        );
        tags_map.insert_one(facet_id("vibe"), plain_tag("Dark"));
        tags_map.insert_one(
            facet_id("played@20241031"),
            PlainTag {
                label: None,
                ..Default::default()
            },
        );
        tags_map
    }

    fn roundtrip_gig_tags(test_file: &TestFile, encode_gigtags: &TagFacetId) {
        let import_config = ImportTrackConfig::default();
        let export_config = ExportTrackConfig {
            encode_gigtags: Some(encode_gigtags.clone()),
            ..Default::default()
        };
        roundtrip(
            test_file,
            &import_config,
            &export_config,
            gig_tags_map().canonicalize_into(),
            modified_gig_tags_map().canonicalize_into(),
        );
    }

    fn roundtrip_only_gig_tags(test_file: &TestFile, encode_gigtags: &TagFacetId) {
        let import_config = ImportTrackConfig::default();
        let export_config = ExportTrackConfig {
            encode_gigtags: Some(encode_gigtags.clone()),
            ..Default::default()
        };
        let mut tags_map = TagsMap::default();
        insert_gig_tags(&mut tags_map);
        roundtrip(
            test_file,
            &import_config,
            &export_config,
            tags_map.canonicalize_into(),
            Default::default(),
        );
    }

    /// Verify that the gig tags are actually encoded into the selected file tag.
    fn encode_gig_tags_into_file_tag(test_file: &TestFile, encode_gigtags: &TagFacetId) {
        let mut temp_file = test_file.copy_to_temp_file();
        let mut track = import_new_track(test_file, temp_file.path(), &Default::default());
        let mut tags_map = TagsMap::default();
        tags_map.insert_one(FACET_ID_GENRE, plain_tag("Techno"));
        insert_gig_tags(&mut tags_map);
        track.tags = tags_map.canonicalize_into();
        let export_config = ExportTrackConfig {
            encode_gigtags: Some(encode_gigtags.clone()),
            ..Default::default()
        };
        export_track(test_file, &mut temp_file, &export_config, &mut track);

        // Import the raw file tags without decoding gig tags.
        let mut import_config = ImportTrackConfig::default();
        import_config
            .flags
            .remove(ImportTrackFlags::GIGTAGS_CGRP | ImportTrackFlags::GIGTAGS_COMM);
        let track = import_new_track(test_file, temp_file.path(), &import_config);
        let tags_map = TagsMap::from(track.tags.untie());
        assert_eq!(2, tags_map.facet_ids().count());
        assert_eq!(
            Some([plain_tag("Techno")].as_slice()),
            tags_map.find(FACET_ID_GENRE).map(|(_, tags)| tags)
        );
        let (_, encoded_tags) = tags_map.find(encode_gigtags).unwrap();
        let [encoded_tag] = encoded_tags else {
            panic!("expected a single encoded tag: {encoded_tags:?}");
        };
        assert_eq!(
            Some(
                "#Plain%20Tag ?s=0.75#Scored energy?s=0.8 vibe?s=0.5#Uplifting vibe#Warm played@20240315"
            ),
            encoded_tag.label.as_ref().map(TagLabel::as_str)
        );
    }

    #[test]
    fn encode_gig_tags_into_file_tag_mp3() {
        encode_gig_tags_into_file_tag(&MP3, FACET_ID_GROUPING);
        encode_gig_tags_into_file_tag(&MP3, FACET_ID_COMMENT);
    }

    #[test]
    fn encode_gig_tags_into_file_tag_m4a() {
        encode_gig_tags_into_file_tag(&M4A, FACET_ID_GROUPING);
        encode_gig_tags_into_file_tag(&M4A, FACET_ID_COMMENT);
    }

    #[test]
    fn encode_gig_tags_into_file_tag_flac() {
        encode_gig_tags_into_file_tag(&FLAC, FACET_ID_GROUPING);
        encode_gig_tags_into_file_tag(&FLAC, FACET_ID_COMMENT);
    }

    #[test]
    fn roundtrip_gig_tags_in_grouping_mp3() {
        roundtrip_gig_tags(&MP3, FACET_ID_GROUPING);
        roundtrip_only_gig_tags(&MP3, FACET_ID_GROUPING);
    }

    #[test]
    fn roundtrip_gig_tags_in_grouping_m4a() {
        roundtrip_gig_tags(&M4A, FACET_ID_GROUPING);
        roundtrip_only_gig_tags(&M4A, FACET_ID_GROUPING);
    }

    #[test]
    fn roundtrip_gig_tags_in_grouping_flac() {
        roundtrip_gig_tags(&FLAC, FACET_ID_GROUPING);
        roundtrip_only_gig_tags(&FLAC, FACET_ID_GROUPING);
    }

    #[test]
    fn roundtrip_gig_tags_in_comment_mp3() {
        roundtrip_gig_tags(&MP3, FACET_ID_COMMENT);
        roundtrip_only_gig_tags(&MP3, FACET_ID_COMMENT);
    }

    #[test]
    fn roundtrip_gig_tags_in_comment_m4a() {
        roundtrip_gig_tags(&M4A, FACET_ID_COMMENT);
        roundtrip_only_gig_tags(&M4A, FACET_ID_COMMENT);
    }

    #[test]
    fn roundtrip_gig_tags_in_comment_flac() {
        roundtrip_gig_tags(&FLAC, FACET_ID_COMMENT);
        roundtrip_only_gig_tags(&FLAC, FACET_ID_COMMENT);
    }
}
