// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
        state: State::ReadWrite,
        source: None,
        start: Default::default(),
        end: None,
        tempo: Default::default(),
        timing: Default::default(),
        start_beat: 0u16,
    }
}

#[test]
fn valid_markers() {
    let mk1 = Marker {
        tempo: TempoBpm(1f64),
        ..base_marker()
    };
    assert!(mk1.is_valid());
    let mk2 = Marker {
        timing: TimeSignature {
            top: 4,
            bottom: 0,
        },
        ..base_marker()
    };
    assert!(mk2.is_valid());
}

#[test]
fn invalid_markers() {
    assert!(!base_marker().is_valid());
    let mk1 = Marker {
        timing: TimeSignature {
            top: 4,
            bottom: 4,
        },
        start_beat: 5,
        ..base_marker()
    };
    assert!(!mk1.is_valid());
}
