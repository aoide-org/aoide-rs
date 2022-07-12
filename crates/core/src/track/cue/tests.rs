// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn is_canonical_slice() {
    let default_cue = Cue {
        bank_index: 0,
        slot_index: None,
        in_marker: None,
        out_marker: None,
        kind: None,
        label: None,
        color: None,
        flags: Default::default(),
    };
    let mut cues = vec![
        Cue {
            bank_index: 2,
            ..default_cue.clone()
        },
        Cue {
            bank_index: 1,
            slot_index: Some(2),
            ..default_cue.clone()
        },
        Cue {
            bank_index: 6,
            ..default_cue.clone()
        },
        Cue {
            bank_index: 7,
            ..default_cue.clone()
        },
        Cue {
            bank_index: 1,
            slot_index: Some(1),
            ..default_cue.clone()
        },
        Cue {
            bank_index: 1,
            slot_index: None,
            ..default_cue.clone()
        },
    ];
    assert!(!cues.is_canonical());
    cues.canonicalize();
    assert!(cues.is_canonical());
    assert_eq!(
        vec![
            Cue {
                bank_index: 1,
                slot_index: None,
                ..default_cue.clone()
            },
            Cue {
                bank_index: 1,
                slot_index: Some(1),
                ..default_cue.clone()
            },
            Cue {
                bank_index: 1,
                slot_index: Some(2),
                ..default_cue.clone()
            },
            Cue {
                bank_index: 2,
                ..default_cue.clone()
            },
            Cue {
                bank_index: 6,
                ..default_cue.clone()
            },
            Cue {
                bank_index: 7,
                ..default_cue
            },
        ],
        cues
    );
}
