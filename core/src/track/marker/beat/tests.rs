// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

fn base_marker() -> Marker {
    Marker {
        position: Default::default(),
        tempo: None,
        timing: None,
        beat_in_bar: None,
        bar_in_phrase: None,
        beat_count: None,
        bar_count: None,
    }
}

#[test]
fn valid_markers() {
    let mk1 = Marker {
        tempo: Some(TempoBpm(1f64)),
        ..base_marker()
    };
    assert!(mk1.is_valid());
    let mk2 = Marker {
        timing: Some(TimeSignature {
            beats_per_bar: 4,
            beat_unit: None,
        }),
        ..base_marker()
    };
    assert!(mk2.is_valid());
}

#[test]
fn invalid_markers() {
    assert!(!base_marker().is_valid());
    let mk1 = Marker {
        timing: Some(TimeSignature {
            beats_per_bar: 4,
            beat_unit: Some(4),
        }),
        beat_in_bar: Some(5),
        ..base_marker()
    };
    assert!(!mk1.is_valid());
}

#[test]
fn uniform_tempo() {
    assert!(!base_marker().is_valid());
    let tempo = Some(TempoBpm(123.0));
    let markers = [
        Marker {
            position: PositionMs(0.0).into(),
            ..base_marker()
        },
        Marker {
            position: PositionMs(1.0).into(),
            tempo,
            ..base_marker()
        },
        Marker {
            position: PositionMs(2.0).into(),
            ..base_marker()
        },
    ];
    assert_eq!(tempo, uniform_tempo_from_markers(markers.iter()));
}

#[test]
fn non_uniform_tempo() {
    assert!(!base_marker().is_valid());
    let markers = [
        Marker {
            position: PositionMs(0.0).into(),
            ..base_marker()
        },
        Marker {
            position: PositionMs(1.0).into(),
            tempo: Some(TempoBpm(123.0)),
            ..base_marker()
        },
        Marker {
            position: PositionMs(2.0).into(),
            ..base_marker()
        },
        Marker {
            position: PositionMs(3.0).into(),
            tempo: Some(TempoBpm(123.1)),
            ..base_marker()
        },
    ];
    assert_eq!(None, uniform_tempo_from_markers(markers.iter()));
}
