// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

struct EntityType;

#[test]
fn default_uid() {
    assert!(EntityUid::default().validate().is_err());
    assert_eq!(EntityUid::default(), EntityUid::nil(),);
}

#[test]
fn default_uid_typed() {
    assert_eq!(
        &EntityUid::default(),
        &*EntityUidTyped::<EntityType>::default(),
    );
}

#[test]
fn clone_uid_typed() {
    let default_uid = EntityUidTyped::<EntityType>::default();
    assert_eq!(default_uid, default_uid.clone(),);
    let random_uid = EntityUid::new();
    println!("random_uid = {random_uid:?}");
    let random_uid_typed = EntityUidTyped::<EntityType>::from_untyped(random_uid);
    // Verify that the typed UID implements std::fmt::Debug
    println!("random_uid_typed = {random_uid_typed:?}");
    assert_eq!(random_uid_typed, random_uid_typed.clone(),);
}

#[test]
fn generate_uid() {
    assert!(EntityUid::new().validate().is_ok());
}

#[test]
fn should_encode_decode_uid() {
    let uid = EntityUid::new();
    let encoded = uid.to_string();
    let decoded = EntityUid::decode_from(&encoded).unwrap();
    assert_eq!(uid, decoded);
}

#[test]
fn should_fail_to_decode_too_long_string() {
    let uid = EntityUid::new();

    // Test encode -> decode roundtrip
    let mut encoded = uid.to_string();
    assert!(EntityUid::decode_from(&encoded).is_ok());

    // Append more characters from the alphabet to the encoded string.
    encoded.push('1');
    assert!(EntityUid::decode_from(&encoded).is_err());
}

#[test]
fn should_fail_to_decode_too_short_string() {
    let uid = EntityUid::new();
    let mut encoded = uid.to_string();
    encoded.truncate(EntityUid::STR_LEN - 1);
    assert!(EntityUid::decode_from(&encoded).is_err());
}

#[test]
fn rev_sequence() {
    let initial = EntityRevision::initial();
    assert!(initial.validate().is_ok());
    let invalid = initial.prev().unwrap();
    assert!(invalid.validate().is_err());

    let next = initial.next().unwrap();
    assert!(next.validate().is_ok());
    assert_ne!(EntityRevision::initial(), next);
    assert_eq!(EntityRevision::initial(), next.prev().unwrap());
    assert!(initial < next);

    let nextnext = next.next().unwrap();
    assert!(nextnext.validate().is_ok());
    assert_ne!(EntityRevision::initial(), next);
    assert!(next < nextnext);
}

#[test]
fn hdr_without_uid() {
    let hdr = EntityHeader::initial_with_uid(EntityUid::default());
    assert!(hdr.validate().is_err());
    assert_eq!(EntityRevision::initial(), hdr.rev);
}

#[test]
fn should_generate_unique_initial_hdrs() {
    let hdr1 = EntityHeader::initial_random();
    let hdr2 = EntityHeader::initial_random();
    assert!(hdr1.validate().is_ok());
    assert_eq!(EntityRevision::initial(), hdr1.rev);
    assert!(hdr2.validate().is_ok());
    assert_eq!(EntityRevision::initial(), hdr2.rev);
    assert_ne!(hdr1.uid, hdr2.uid);
    assert_eq!(hdr1.rev, hdr2.rev);
}
