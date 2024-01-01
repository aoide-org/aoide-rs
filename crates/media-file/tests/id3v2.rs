// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{io::BufReader, path::Path};

use aoide_core::{
    media::content::ContentLink, music::tempo::TempoBpm, util::clock::OffsetDateTimeMs, Track,
};
use aoide_media_file::{
    io::{
        export::export_track_to_file,
        import::{import_into_track, ImportTrack, Reader},
    },
    util::guess_mime_from_file_path,
};
use lofty::{FileType, ItemKey, Probe, Tag, TagExt as _, TagType, TaggedFileExt as _};
use mime::Mime;
use tempfile::NamedTempFile;

fn copy_named_temp_file<T: AsRef<Path>>(file_path: T) -> NamedTempFile {
    let temp_file = ::tempfile::NamedTempFile::new().unwrap();
    std::fs::copy(file_path, temp_file.path()).unwrap();
    temp_file
}

fn import_new_track_from_file_path<T: AsRef<Path>>(
    file_path: T,
    content_type: Option<Mime>,
) -> Track {
    let import_track = ImportTrack::NewTrack {
        collected_at: OffsetDateTimeMs::now_utc(),
    };
    let content_type = content_type
        .or_else(|| guess_mime_from_file_path(file_path.as_ref()).ok())
        .unwrap();
    let content_link = ContentLink {
        path: Default::default(),
        rev: None,
    };
    let mut track = import_track.with_content(content_link, content_type);
    let file = Box::new(std::fs::File::open(file_path.as_ref()).unwrap());
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let issues = import_into_track(&mut reader, &Default::default(), &mut track).unwrap();
    assert!(issues.is_empty());
    track
}

#[test]
#[allow(clippy::float_cmp)]
fn integer_bpm_roundtrip() {
    let mut file = copy_named_temp_file("tests/assets/empty.mp3");

    {
        let tagged_file = Probe::new(&mut file.as_file())
            .guess_file_type()
            .unwrap()
            .read()
            .unwrap();
        assert_eq!(FileType::Mpeg, tagged_file.file_type());
        assert!(tagged_file.tags().is_empty());
    }

    let bpm = 123;

    let mut tag = Tag::new(TagType::Id3v2);
    tag.insert_text(ItemKey::Bpm, bpm.to_string());
    tag.save_to_path(file.path()).unwrap();

    let mut track =
        import_new_track_from_file_path(file.path(), Some("audio/mpeg".parse().unwrap()));
    assert!(track
        .metrics
        .flags
        .contains(aoide_core::track::metric::MetricsFlags::TEMPO_BPM_NON_FRACTIONAL));
    assert_eq!(f64::from(bpm), track.metrics.tempo_bpm.unwrap().value());

    // Write the unmodified track metadata back to the file.
    export_track_to_file(
        file.as_file_mut(),
        Some("mp3"),
        &Default::default(),
        &mut track,
        None,
    )
    .unwrap();
    assert!(track
        .metrics
        .flags
        .contains(aoide_core::track::metric::MetricsFlags::TEMPO_BPM_NON_FRACTIONAL));
    assert_eq!(f64::from(bpm), track.metrics.tempo_bpm.unwrap().value());

    // Verify that the bpm didn't change
    let mut track =
        import_new_track_from_file_path(file.path(), Some("audio/mpeg".parse().unwrap()));
    assert!(track
        .metrics
        .flags
        .contains(aoide_core::track::metric::MetricsFlags::TEMPO_BPM_NON_FRACTIONAL));
    assert_eq!(f64::from(bpm), track.metrics.tempo_bpm.unwrap().value());

    // Modify and write the track metadata back to the file.
    let fractional_bpm = TempoBpm::new(123.5);
    track.metrics.tempo_bpm = Some(fractional_bpm);
    export_track_to_file(
        file.as_file_mut(),
        Some("mp3"),
        &Default::default(),
        &mut track,
        None,
    )
    .unwrap();

    // Verify that the fractional bpm has been written to the file.
    let track = import_new_track_from_file_path(file.path(), Some("audio/mpeg".parse().unwrap()));
    assert!(!track
        .metrics
        .flags
        .contains(aoide_core::track::metric::MetricsFlags::TEMPO_BPM_NON_FRACTIONAL));
    assert_eq!(fractional_bpm, track.metrics.tempo_bpm.unwrap());
}
