// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use semval::Validate as _;

use super::{Uuid, UuidEncodedStr};

#[test]
fn nil_str() {
    assert_eq!(
        UuidEncodedStr::from(Uuid::NIL).as_str(),
        Uuid::NIL.to_string().as_str(),
    );
}

#[test]
fn default() {
    assert!(Uuid::default().validate().is_err());
    assert_eq!(Uuid::default(), Uuid::NIL);
}

#[test]
fn random() {
    assert!(Uuid::random().validate().is_ok());
}

#[test]
fn should_encode_decode_uuid() {
    let uuid = Uuid::random();
    let encoded = uuid.to_string();
    assert_eq!(encoded.len(), Uuid::STR_LEN);
    let decoded = Uuid::decode_str(&encoded).unwrap();
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
