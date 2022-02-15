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

use aoide_core::util::color::RgbColor as CoreRgbColor;

#[cfg(feature = "with-schemars")]
#[test]
fn generate_json_schema() {
    let json_schema = schemars::schema_for!(Color);
    let schema_string = serde_json::to_string_pretty(&json_schema).unwrap();
    println!("{}", schema_string);
    assert!(!schema_string.is_empty());
}

#[test]
fn deserialize_json() {
    assert_eq!(
        RgbColor::from(CoreRgbColor::BLACK),
        serde_json::from_str("\"#000000\"").unwrap()
    );
    assert_eq!(
        RgbColor::from(CoreRgbColor::WHITE),
        serde_json::from_str("\"#FfFfFf\"").unwrap()
    );
    assert_eq!(
        RgbColor::from(CoreRgbColor::RED),
        serde_json::from_str("\"#FF0000\"").unwrap()
    );
    assert_eq!(
        RgbColor::from(CoreRgbColor::GREEN),
        serde_json::from_str("\"#00ff00\"").unwrap()
    );
    assert_eq!(
        RgbColor::from(CoreRgbColor::BLUE),
        serde_json::from_str("\"#0000fF\"").unwrap()
    );
    assert_eq!(
        RgbColor::from(CoreRgbColor(0xabcdef)),
        serde_json::from_str("\"#aBcDeF\"").unwrap()
    );
}

#[test]
fn deserialize_json_leading_whitespace() {
    assert!(serde_json::from_str::<RgbColor>("\" #000000\"").is_err());
}

#[test]
fn deserialize_json_trailing_whitespace() {
    assert!(serde_json::from_str::<RgbColor>("\"#000000 \"").is_err());
}

#[test]
fn deserialize_json_whitespace() {
    assert!(serde_json::from_str::<RgbColor>("\"#000 000\"").is_err());
    assert!(serde_json::from_str::<RgbColor>("\"# 000000\"").is_err());
}

#[test]
fn deserialize_json_invalid_hex_digits() {
    assert!(serde_json::from_str::<RgbColor>("\"#g00000\"").is_err());
    assert!(serde_json::from_str::<RgbColor>("\"#00 000\"").is_err());
    assert!(serde_json::from_str::<RgbColor>("\"#00_00_00\"").is_err());
    assert!(serde_json::from_str::<RgbColor>("\"#000_000\"").is_err());
}

#[test]
fn deserialize_json_invalid_prefix() {
    assert!(serde_json::from_str::<RgbColor>("\"##000000\"").is_err());
}

#[test]
fn deserialize_json_invalid_suffix() {
    assert!(serde_json::from_str::<RgbColor>("\"#000000#\"").is_err());
    assert!(serde_json::from_str::<RgbColor>("\"#00000##\"").is_err());
}

#[test]
#[test]
fn deserialize_json_too_long() {
    assert!(serde_json::from_str::<RgbColor>("\"#0000000\"").is_err());
}

#[test]
fn deserialize_json_too_short() {
    assert!(serde_json::from_str::<RgbColor>("\"#00000\"").is_err());
}

#[test]
fn serialize_json() {
    assert_eq!(
        "\"#000000\"",
        serde_json::to_string(&RgbColor::from(CoreRgbColor::BLACK)).unwrap()
    );
    assert_eq!(
        "\"#FFFFFF\"",
        serde_json::to_string(&RgbColor::from(CoreRgbColor::WHITE)).unwrap()
    );
    assert_eq!(
        "\"#FF0000\"",
        serde_json::to_string(&RgbColor::from(CoreRgbColor::RED)).unwrap()
    );
    assert_eq!(
        "\"#00FF00\"",
        serde_json::to_string(&RgbColor::from(CoreRgbColor::GREEN)).unwrap()
    );
    assert_eq!(
        "\"#0000FF\"",
        serde_json::to_string(&RgbColor::from(CoreRgbColor::BLUE)).unwrap()
    );
}
