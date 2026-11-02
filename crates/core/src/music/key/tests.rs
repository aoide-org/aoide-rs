// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#[test]
fn format_parse_roundtrip() {
    for code in KeyCode::iter() {
        assert_eq!(
            Some(code),
            KeyCode::try_from_canonical_str(code.as_canonical_str()),
            "canonical"
        );
        assert_eq!(
            Some(code),
            KeyCode::try_from_traditional_str(code.as_traditional_str()),
            "traditional"
        );
        assert_eq!(
            Some(code),
            KeyCode::try_from_traditional_ascii_str(code.as_traditional_ascii_str()),
            "traditional ASCII"
        );
        assert_eq!(
            Some(code),
            KeyCode::try_from_openkey_str(code.as_openkey_str()),
            "OpenKey"
        );
        assert_eq!(
            Some(code),
            KeyCode::try_from_lancelot_str(code.as_lancelot_str()),
            "Lancelot"
        );
        assert_eq!(
            Some(code),
            KeyCode::try_from_traxsource_str(code.as_traxsource_str()),
            "Traxsource"
        );
        assert_eq!(
            Some(code),
            KeyCode::try_from_beatport_str(code.as_beatport_str()),
            "Beatport"
        );
        assert_eq!(
            Some(code),
            KeyCode::try_from_serato_str(code.as_serato_str()),
            "Serato"
        );
    }
}
