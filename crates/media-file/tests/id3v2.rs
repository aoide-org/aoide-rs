// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{io::BufReader, path::Path};

use aoide_core::{
    Track, media::content::ContentLink, music::tempo::TempoBpm, util::clock::OffsetDateTimeMs,
};
use aoide_media_file::{
    io::{
        export::export_track_to_file_with_type,
        import::{ImportTrack, Reader, import_into_track},
    },
    util::guess_mime_from_file_path,
};
use lofty::{
    config::WriteOptions,
    file::{FileType, TaggedFileExt as _},
    probe::Probe,
    tag::{ItemKey, Tag, TagExt as _, TagType},
};
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
    if !issues.is_empty() {
        println!("Import issues: {issues:?}");
    }
    assert!(issues.is_empty());
    track
}

#[test]
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

    let fractional_bpm = TempoBpm::new(122.9);
    let integer_bpm = TempoBpm::new(fractional_bpm.value().round());
    assert_ne!(fractional_bpm, integer_bpm);

    let mut tag = Tag::new(TagType::Id3v2);
    tag.insert_text(
        ItemKey::IntegerBpm,
        (integer_bpm.value() as i32).to_string(),
    );
    tag.save_to_path(file.path(), WriteOptions::default())
        .unwrap();

    let mut track =
        import_new_track_from_file_path(file.path(), Some("audio/mpeg".parse().unwrap()));
    assert!(
        track
            .metrics
            .flags
            .contains(aoide_core::track::metric::MetricsFlags::TEMPO_BPM_INTEGER)
    );
    assert_eq!(integer_bpm, track.metrics.tempo_bpm.unwrap());

    // Set a fractional BPM and write the track metadata back to the file.
    track.metrics.tempo_bpm = Some(fractional_bpm);
    track
        .metrics
        .flags
        .remove(aoide_core::track::metric::MetricsFlags::TEMPO_BPM_INTEGER);
    export_track_to_file_with_type(
        file.as_file_mut(),
        FileType::Mpeg,
        &Default::default(),
        &mut track,
        None,
    )
    .unwrap();

    // Verify that the fractional BPM is imported from the custom TXXX BPM tag
    // instead of the imprecise integer BPM from the standard tag.
    let track = import_new_track_from_file_path(file.path(), Some("audio/mpeg".parse().unwrap()));
    assert!(
        !track
            .metrics
            .flags
            .contains(aoide_core::track::metric::MetricsFlags::TEMPO_BPM_INTEGER)
    );
    assert_eq!(fractional_bpm, track.metrics.tempo_bpm.unwrap());
}

/// Create a minimal, untagged AIFF file with a few frames of silence.
fn new_aiff_temp_file() -> NamedTempFile {
    const CHANNELS: u16 = 1;
    const FRAMES: u32 = 4;
    const SAMPLE_SIZE_BITS: u16 = 16;
    // 44100 Hz as an 80-bit IEEE 754 extended precision float.
    const SAMPLE_RATE: [u8; 10] = [0x40, 0x0E, 0xAC, 0x44, 0, 0, 0, 0, 0, 0];

    let sound_data_len = FRAMES * u32::from(CHANNELS) * u32::from(SAMPLE_SIZE_BITS / 8);
    let mut comm = Vec::new();
    comm.extend_from_slice(&CHANNELS.to_be_bytes());
    comm.extend_from_slice(&FRAMES.to_be_bytes());
    comm.extend_from_slice(&SAMPLE_SIZE_BITS.to_be_bytes());
    comm.extend_from_slice(&SAMPLE_RATE);
    let mut ssnd = vec![0; 8]; // offset and block size
    ssnd.resize(ssnd.len() + sound_data_len as usize, 0);

    let mut chunks = Vec::new();
    for (id, data) in [(b"COMM", comm), (b"SSND", ssnd)] {
        chunks.extend_from_slice(id);
        chunks.extend_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
        chunks.extend_from_slice(&data);
    }
    let mut aiff = Vec::new();
    aiff.extend_from_slice(b"FORM");
    aiff.extend_from_slice(&u32::try_from(4 + chunks.len()).unwrap().to_be_bytes());
    aiff.extend_from_slice(b"AIFF");
    aiff.extend_from_slice(&chunks);

    let temp_file = ::tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), aiff).unwrap();
    temp_file
}

#[test]
fn aiff_fractional_bpm_roundtrip() {
    let mut file = new_aiff_temp_file();
    let content_type: Mime = "audio/aiff".parse().unwrap();

    {
        let tagged_file = Probe::new(&mut file.as_file())
            .guess_file_type()
            .unwrap()
            .read()
            .unwrap();
        assert_eq!(FileType::Aiff, tagged_file.file_type());
        assert!(tagged_file.tags().is_empty());
    }

    let fractional_bpm = TempoBpm::new(122.9);
    let integer_bpm = TempoBpm::new(fractional_bpm.value().round());

    let mut tag = Tag::new(TagType::Id3v2);
    tag.insert_text(
        ItemKey::IntegerBpm,
        (integer_bpm.value() as i32).to_string(),
    );
    tag.save_to_path(file.path(), WriteOptions::default())
        .unwrap();

    let mut track = import_new_track_from_file_path(file.path(), Some(content_type.clone()));
    assert_eq!(integer_bpm, track.metrics.tempo_bpm.unwrap());

    // Set a fractional BPM and write the track metadata back to the file.
    track.metrics.tempo_bpm = Some(fractional_bpm);
    track
        .metrics
        .flags
        .remove(aoide_core::track::metric::MetricsFlags::TEMPO_BPM_INTEGER);
    export_track_to_file_with_type(
        file.as_file_mut(),
        FileType::Aiff,
        &Default::default(),
        &mut track,
        None,
    )
    .unwrap();

    // Verify that the fractional BPM is imported from the custom TXXX BPM tag
    // like for MPEG files.
    let track = import_new_track_from_file_path(file.path(), Some(content_type));
    assert!(
        !track
            .metrics
            .flags
            .contains(aoide_core::track::metric::MetricsFlags::TEMPO_BPM_INTEGER)
    );
    assert_eq!(fractional_bpm, track.metrics.tempo_bpm.unwrap());
}
