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

use aoide_core::util::color::ColorRgb as CoreColorRgb;

#[test]
fn deserialize_json() {
    assert_eq!(ColorRgb::from(CoreColorRgb::BLACK), serde_json::from_str("\"#000000\"").unwrap());
    assert_eq!(ColorRgb::from(CoreColorRgb::WHITE), serde_json::from_str("\"#FfFfFf\"").unwrap());
    assert_eq!(ColorRgb::from(CoreColorRgb::RED), serde_json::from_str("\"#FF0000\"").unwrap());
    assert_eq!(ColorRgb::from(CoreColorRgb::GREEN), serde_json::from_str("\"#00ff00\"").unwrap());
    assert_eq!(ColorRgb::from(CoreColorRgb::BLUE), serde_json::from_str("\"#0000fF\"").unwrap());
    assert_eq!(ColorRgb::from(CoreColorRgb(0xabcdef)), serde_json::from_str("\"#aBcDeF\"").unwrap());
}

#[test]
fn deserialize_json_leading_whitespace() {
    assert!(serde_json::from_str::<ColorRgb>("\" #000000\"").is_err());
}

#[test]
fn deserialize_json_trailing_whitespace() {
    assert!(serde_json::from_str::<ColorRgb>("\"#000000 \"").is_err());
}

#[test]
fn deserialize_json_whitespace() {
    assert!(serde_json::from_str::<ColorRgb>("\"#000 000\"").is_err());
    assert!(serde_json::from_str::<ColorRgb>("\"# 000000\"").is_err());
}

#[test]
fn deserialize_json_invalid_hex_digits() {
    assert!(serde_json::from_str::<ColorRgb>("\"#g00000\"").is_err());
    assert!(serde_json::from_str::<ColorRgb>("\"#00 000\"").is_err());
    assert!(serde_json::from_str::<ColorRgb>("\"#00_00_00\"").is_err());
    assert!(serde_json::from_str::<ColorRgb>("\"#000_000\"").is_err());
}

#[test]
fn deserialize_json_invalid_prefix() {
    assert!(serde_json::from_str::<ColorRgb>("\"##000000\"").is_err());
}

#[test]
fn deserialize_json_invalid_suffix() {
    assert!(serde_json::from_str::<ColorRgb>("\"#000000#\"").is_err());
    assert!(serde_json::from_str::<ColorRgb>("\"#00000##\"").is_err());
}

#[test]
#[test]
fn deserialize_json_too_long() {
    assert!(serde_json::from_str::<ColorRgb>("\"#0000000\"").is_err());
}

#[test]
fn deserialize_json_too_short() {
    assert!(serde_json::from_str::<ColorRgb>("\"#00000\"").is_err());
}

#[test]
fn serialize_json() {
    assert_eq!("\"#000000\"", serde_json::to_string(&ColorRgb::from(CoreColorRgb::BLACK)).unwrap());
    assert_eq!("\"#FFFFFF\"", serde_json::to_string(&ColorRgb::from(CoreColorRgb::WHITE)).unwrap());
    assert_eq!("\"#FF0000\"", serde_json::to_string(&ColorRgb::from(CoreColorRgb::RED)).unwrap());
    assert_eq!("\"#00FF00\"", serde_json::to_string(&ColorRgb::from(CoreColorRgb::GREEN)).unwrap());
    assert_eq!("\"#0000FF\"", serde_json::to_string(&ColorRgb::from(CoreColorRgb::BLUE)).unwrap());
}
