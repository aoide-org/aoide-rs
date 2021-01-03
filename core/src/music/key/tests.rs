// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
fn invalid_key_code() {
    assert!(KeySignature(KeySignature::min_code() - 1)
        .validate()
        .is_err());
    assert!(KeySignature(KeySignature::max_code() + 1)
        .validate()
        .is_err());
}

#[test]
fn convert_key_sigs() {
    // C maj
    assert_eq!(
        KeySignature::from_code(1),
        OpenKeySignature::new(1, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::from_code(1),
        LancelotKeySignature::new(8, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::from_code(1),
        EngineKeySignature::from_code(24).into()
    );
    // A min
    assert_eq!(
        KeySignature::from_code(2),
        OpenKeySignature::new(1, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::from_code(2),
        LancelotKeySignature::new(8, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::from_code(2),
        EngineKeySignature::from_code(1).into()
    );
    // E maj
    assert_eq!(
        KeySignature::from_code(9),
        OpenKeySignature::new(5, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::from_code(9),
        LancelotKeySignature::new(12, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::from_code(9),
        EngineKeySignature::from_code(8).into()
    );
    // Db min
    assert_eq!(
        KeySignature::from_code(10),
        OpenKeySignature::new(5, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::from_code(10),
        LancelotKeySignature::new(12, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::from_code(10),
        EngineKeySignature::from_code(9).into()
    );
    // B maj
    assert_eq!(
        KeySignature::from_code(11),
        OpenKeySignature::new(6, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::from_code(11),
        LancelotKeySignature::new(1, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::from_code(11),
        EngineKeySignature::from_code(10).into()
    );
    // Ab min
    assert_eq!(
        KeySignature::from_code(12),
        OpenKeySignature::new(6, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::from_code(12),
        LancelotKeySignature::new(1, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::from_code(12),
        EngineKeySignature::from_code(11).into()
    );
    // F maj
    assert_eq!(
        KeySignature::from_code(23),
        OpenKeySignature::new(12, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::from_code(23),
        LancelotKeySignature::new(7, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::from_code(23),
        EngineKeySignature::from_code(22).into()
    );
    // D min
    assert_eq!(
        KeySignature::from_code(24),
        OpenKeySignature::new(12, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::from_code(24),
        LancelotKeySignature::new(7, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::from_code(24),
        EngineKeySignature::from_code(23).into()
    );
}

#[test]
fn display_key_sigs() {
    assert_eq!(
        "1d",
        OpenKeySignature::from(KeySignature::from_code(1)).to_string()
    ); // C maj
    assert_eq!(
        "8B",
        LancelotKeySignature::from(KeySignature::from_code(1)).to_string()
    ); // C maj
    assert_eq!(
        "1m",
        OpenKeySignature::from(KeySignature::from_code(2)).to_string()
    ); // A min
    assert_eq!(
        "8A",
        LancelotKeySignature::from(KeySignature::from_code(2)).to_string()
    ); // A min
    assert_eq!(
        "12d",
        OpenKeySignature::from(KeySignature::from_code(23)).to_string()
    ); // F maj
    assert_eq!(
        "7B",
        LancelotKeySignature::from(KeySignature::from_code(23)).to_string()
    ); // F maj
    assert_eq!(
        "12m",
        OpenKeySignature::from(KeySignature::from_code(24)).to_string()
    ); // D min
    assert_eq!(
        "7A",
        LancelotKeySignature::from(KeySignature::from_code(24)).to_string()
    ); // D min
}
