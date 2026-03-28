// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

struct EntityType;

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
    assert_eq!(default_uid, default_uid.clone());
    let random_uid = EntityUid::random();
    println!("random_uid = {random_uid:?}");
    let random_uid_typed = EntityUidTyped::<EntityType>::from_untyped(random_uid);
    // Verify that the typed UID implements std::fmt::Debug
    println!("random_uid_typed = {random_uid_typed:?}");
    assert_eq!(random_uid_typed, random_uid_typed.clone());
}

#[test]
fn rev_sequence() {
    let initial = EntityRevision::INITIAL;
    assert!(initial.validate().is_ok());
    let invalid = initial.prev().unwrap();
    assert!(invalid.validate().is_err());

    let next = initial.next().unwrap();
    assert!(next.validate().is_ok());
    assert_ne!(EntityRevision::INITIAL, next);
    assert_eq!(EntityRevision::INITIAL, next.prev().unwrap());
    assert!(initial < next);

    let nextnext = next.next().unwrap();
    assert!(nextnext.validate().is_ok());
    assert_ne!(EntityRevision::INITIAL, next);
    assert!(next < nextnext);
}

#[test]
fn hdr_without_uid() {
    let hdr = EntityHeader::initial_with_uid(EntityUid::default());
    assert!(hdr.validate().is_err());
    assert_eq!(EntityRevision::INITIAL, hdr.rev);
}

#[test]
fn should_generate_unique_initial_hdrs() {
    let hdr1 = EntityHeader::initial_random();
    let hdr2 = EntityHeader::initial_random();
    assert!(hdr1.validate().is_ok());
    assert_eq!(EntityRevision::INITIAL, hdr1.rev);
    assert!(hdr2.validate().is_ok());
    assert_eq!(EntityRevision::INITIAL, hdr2.rev);
    assert_ne!(hdr1.uid, hdr2.uid);
    assert_eq!(hdr1.rev, hdr2.rev);
}

#[test]
fn default_entity_revision() {
    assert_eq!(EntityRevision::RESERVED_DEFAULT, EntityRevision::default());
    assert!(!EntityRevision::RESERVED_DEFAULT.is_valid());
}

#[test]
fn initial_entity_revision() {
    assert!(EntityRevision::INITIAL.is_valid());
    assert!(!EntityRevision::INITIAL.prev().unwrap().is_valid());
    assert!(EntityRevision::INITIAL.next().unwrap().is_valid());
}
