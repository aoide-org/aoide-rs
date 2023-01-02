// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io::BufReader;

use crate::util::guess_mime_from_path;

use super::*;

#[test]
fn import_replay_gain_valid() {
    let mut importer = Importer::new();
    assert_eq!(
        Some(LoudnessLufs(-8.49428)),
        importer.import_loudness_from_replay_gain("-9.50572 dB")
    );
    assert_eq!(
        Some(LoudnessLufs(-8.49428)),
        importer.import_loudness_from_replay_gain(" -9.50572db ")
    );
    assert_eq!(
        Some(LoudnessLufs(-18.178062)),
        importer.import_loudness_from_replay_gain("0.178062 DB")
    );
    assert_eq!(
        Some(LoudnessLufs(-18.178062)),
        importer.import_loudness_from_replay_gain("  +0.178062   dB ")
    );
    assert!(importer.finish().into_messages().is_empty());
}

#[test]
fn import_replay_gain_invalid() {
    let mut importer = Importer::new();
    assert!(importer
        .import_loudness_from_replay_gain("-9.50572")
        .is_none());
    assert!(importer
        .import_loudness_from_replay_gain("- 9.50572 dB")
        .is_none());
    assert!(importer
        .import_loudness_from_replay_gain("+ 0.178062 dB")
        .is_none());
    assert!(importer
        .import_loudness_from_replay_gain("+0.178062")
        .is_none());
}

#[test]
#[cfg(feature = "fmt-mp3")]
#[ignore] // a hack for debugging purposes
fn import_tmp_test_mp3() {
    let path = Path::new("/tmp/test.mp3");
    let file = Box::new(std::fs::File::open(path).unwrap());
    let mut reader: Box<dyn Reader> = Box::new(BufReader::new(file));
    let input = NewTrackInput {
        collected_at: aoide_core::util::clock::DateTime::now_utc(),
        content_rev: None,
    };
    let mime = guess_mime_from_path(path).unwrap();
    let mut track = input.into_new_track(Default::default(), mime);
    let issues = import_into_track(&mut reader, &Default::default(), &mut track).unwrap();
    assert!(issues.is_empty());
}
