// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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
fn default_time_signature() {
    assert!(!TimeSignature::default().is_valid());
}

#[test]
fn default_key_signature() {
    assert!(!KeySignature::default().is_valid());
    assert!(!OpenKeySignature::default().is_valid());
    assert!(!LancelotKeySignature::default().is_valid());
}

#[test]
fn convert_key_signatures() {
    // C maj
    assert_eq!(
        KeySignature::new(1),
        OpenKeySignature::new(1, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(1),
        LancelotKeySignature::new(8, KeyMode::Major).into()
    );
    assert_eq!(KeySignature::new(1), EngineKeySignature::new(24).into());
    // A min
    assert_eq!(
        KeySignature::new(2),
        OpenKeySignature::new(1, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(2),
        LancelotKeySignature::new(8, KeyMode::Minor).into()
    );
    assert_eq!(KeySignature::new(2), EngineKeySignature::new(1).into());
    // E maj
    assert_eq!(
        KeySignature::new(9),
        OpenKeySignature::new(5, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(9),
        LancelotKeySignature::new(12, KeyMode::Major).into()
    );
    assert_eq!(KeySignature::new(9), EngineKeySignature::new(8).into());
    // Db min
    assert_eq!(
        KeySignature::new(10),
        OpenKeySignature::new(5, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(10),
        LancelotKeySignature::new(12, KeyMode::Minor).into()
    );
    assert_eq!(KeySignature::new(10), EngineKeySignature::new(9).into());
    // B maj
    assert_eq!(
        KeySignature::new(11),
        OpenKeySignature::new(6, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(11),
        LancelotKeySignature::new(1, KeyMode::Major).into()
    );
    assert_eq!(KeySignature::new(11), EngineKeySignature::new(10).into());
    // Ab min
    assert_eq!(
        KeySignature::new(12),
        OpenKeySignature::new(6, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(12),
        LancelotKeySignature::new(1, KeyMode::Minor).into()
    );
    assert_eq!(KeySignature::new(12), EngineKeySignature::new(11).into());
    // F maj
    assert_eq!(
        KeySignature::new(23),
        OpenKeySignature::new(12, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(23),
        LancelotKeySignature::new(7, KeyMode::Major).into()
    );
    assert_eq!(KeySignature::new(23), EngineKeySignature::new(22).into());
    // D min
    assert_eq!(
        KeySignature::new(24),
        OpenKeySignature::new(12, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(24),
        LancelotKeySignature::new(7, KeyMode::Minor).into()
    );
    assert_eq!(KeySignature::new(24), EngineKeySignature::new(23).into());
}

#[test]
fn display_key_signatures() {
    assert_eq!(
        "1d",
        format!("{}", OpenKeySignature::from(KeySignature::new(1)))
    ); // C maj
    assert_eq!(
        "8B",
        format!("{}", LancelotKeySignature::from(KeySignature::new(1)))
    ); // C maj
    assert_eq!(
        "1m",
        format!("{}", OpenKeySignature::from(KeySignature::new(2)))
    ); // A min
    assert_eq!(
        "8A",
        format!("{}", LancelotKeySignature::from(KeySignature::new(2)))
    ); // A min
    assert_eq!(
        "12d",
        format!("{}", OpenKeySignature::from(KeySignature::new(23)))
    ); // F maj
    assert_eq!(
        "7B",
        format!("{}", LancelotKeySignature::from(KeySignature::new(23)))
    ); // F maj
    assert_eq!(
        "12m",
        format!("{}", OpenKeySignature::from(KeySignature::new(24)))
    ); // D min
    assert_eq!(
        "7A",
        format!("{}", LancelotKeySignature::from(KeySignature::new(24)))
    ); // D min
}
