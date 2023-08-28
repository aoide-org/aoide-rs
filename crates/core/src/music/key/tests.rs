// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use strum::IntoEnumIterator as _;

use super::*;

#[test]
fn from_to_value() {
    for key_code in KeyCode::iter() {
        assert_eq!(
            key_code,
            KeyCode::try_from_value(key_code.to_value()).unwrap()
        );
    }
}

#[test]
fn convert_key_sigs() {
    assert_eq!(
        KeySignature::new(KeyCode::Cmaj),
        OpenKeySignature::new(1, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Cmaj),
        LancelotKeySignature::new(8, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Cmaj),
        EngineKeySignature::from_code(24).into()
    );

    assert_eq!(
        KeySignature::new(KeyCode::Amin),
        OpenKeySignature::new(1, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Amin),
        LancelotKeySignature::new(8, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Amin),
        EngineKeySignature::from_code(1).into()
    );

    assert_eq!(
        KeySignature::new(KeyCode::Emaj),
        OpenKeySignature::new(5, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Emaj),
        LancelotKeySignature::new(12, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Emaj),
        EngineKeySignature::from_code(8).into()
    );

    assert_eq!(
        KeySignature::new(KeyCode::Dbmin),
        OpenKeySignature::new(5, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Dbmin),
        LancelotKeySignature::new(12, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Dbmin),
        EngineKeySignature::from_code(9).into()
    );

    assert_eq!(
        KeySignature::new(KeyCode::Bmaj),
        OpenKeySignature::new(6, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Bmaj),
        LancelotKeySignature::new(1, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Bmaj),
        EngineKeySignature::from_code(10).into()
    );

    assert_eq!(
        KeySignature::new(KeyCode::Abmin),
        OpenKeySignature::new(6, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Abmin),
        LancelotKeySignature::new(1, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Abmin),
        EngineKeySignature::from_code(11).into()
    );

    assert_eq!(
        KeySignature::new(KeyCode::Fmaj),
        OpenKeySignature::new(12, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Fmaj),
        LancelotKeySignature::new(7, KeyMode::Major).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Fmaj),
        EngineKeySignature::from_code(22).into()
    );

    assert_eq!(
        KeySignature::new(KeyCode::Dmin),
        OpenKeySignature::new(12, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Dmin),
        LancelotKeySignature::new(7, KeyMode::Minor).into()
    );
    assert_eq!(
        KeySignature::new(KeyCode::Dmin),
        EngineKeySignature::from_code(23).into()
    );
}

#[test]
fn display_key_sigs() {
    assert_eq!(
        "1d",
        OpenKeySignature::from(KeySignature::new(KeyCode::Cmaj)).to_string()
    );
    assert_eq!(
        "8B",
        LancelotKeySignature::from(KeySignature::new(KeyCode::Cmaj)).to_string()
    );

    assert_eq!(
        "1m",
        OpenKeySignature::from(KeySignature::new(KeyCode::Amin)).to_string()
    );
    assert_eq!(
        "8A",
        LancelotKeySignature::from(KeySignature::new(KeyCode::Amin)).to_string()
    );

    assert_eq!(
        "12d",
        OpenKeySignature::from(KeySignature::new(KeyCode::Fmaj)).to_string()
    );
    assert_eq!(
        "7B",
        LancelotKeySignature::from(KeySignature::new(KeyCode::Fmaj)).to_string()
    );

    assert_eq!(
        "12m",
        OpenKeySignature::from(KeySignature::new(KeyCode::Dmin)).to_string()
    );
    assert_eq!(
        "7A",
        LancelotKeySignature::from(KeySignature::new(KeyCode::Dmin)).to_string()
    );
}
