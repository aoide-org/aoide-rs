// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use semval::Validate as _;

use super::{Uuid, UuidEncodedStr};

#[test]
fn nil() {
    assert_eq!(Uuid::NIL.encode_str(), UuidEncodedStr::NIL);
    assert_eq!(UuidEncodedStr::NIL.decode(), Uuid::NIL);
    assert!(Uuid::NIL.validate().is_err());
}

#[test]
fn default() {
    assert_eq!(Uuid::default(), Uuid::NIL);
    assert_eq!(UuidEncodedStr::default(), UuidEncodedStr::NIL);
}

#[test]
fn random_is_valid() {
    assert!(Uuid::random().validate().is_ok());
}

#[test]
fn should_encode_decode_uuid() {
    let uuid = Uuid::random();
    let encoded_str = uuid.encode_str();
    assert_eq!(encoded_str.len(), Uuid::STR_LEN);
    let decoded = encoded_str.decode();
    assert_eq!(uuid, decoded);
}

#[test]
fn should_fail_to_decode_too_long_string() {
    let uuid = Uuid::random();

    // Test encode -> decode roundtrip
    let mut encoded = uuid.to_string();
    assert!(Uuid::decode_str(&encoded).is_ok());

    // Append the first character of the alphabet to the encoded string.
    encoded.push('0');
    assert!(Uuid::decode_str(&encoded).is_err());
}

#[test]
fn should_fail_to_decode_too_short_string() {
    let uuid = Uuid::random();
    let mut encoded = uuid.to_string();
    encoded.truncate(Uuid::STR_LEN - 1);
    assert!(Uuid::decode_str(&encoded).is_err());
}
