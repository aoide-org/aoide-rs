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

fn base_marker(signature: KeySignature) -> Marker {
    Marker {
        start: Default::default(),
        end: None,
        signature,
    }
}

#[test]
fn uniform_key() {
    let signature = KeySignature::from_code(KeySignature::min_code());
    let markers = [
        Marker {
            start: PositionMs(0.0).into(),
            ..base_marker(signature)
        },
        Marker {
            start: PositionMs(1.0).into(),
            ..base_marker(signature)
        },
    ];
    assert_eq!(Some(signature), uniform_key_from_markers(markers.iter()));
}

#[test]
fn non_uniform_key() {
    let markers = [
        Marker {
            start: PositionMs(0.0).into(),
            ..base_marker(KeySignature::from_code(KeySignature::min_code()))
        },
        Marker {
            start: PositionMs(1.0).into(),
            ..base_marker(KeySignature::from_code(KeySignature::max_code()))
        },
    ];
    assert_eq!(None, uniform_key_from_markers(markers.iter()));
}
