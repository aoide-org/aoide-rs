// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::*;

#[test]
fn is_canonical_slice() {
    let default_cue = Cue {
        bank_index: 0,
        slot_index: None,
        in_marker: None,
        out_marker: None,
        color: None,
        label: None,
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
