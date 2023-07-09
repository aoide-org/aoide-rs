// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
        Some(LoudnessLufs(-18.178_062)),
        importer.import_loudness_from_replay_gain("0.178062 DB")
    );
    assert_eq!(
        Some(LoudnessLufs(-18.178_062)),
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
