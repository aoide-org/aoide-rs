// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
fn default_uid() {
    assert!(!EntityUid::default().validate().is_ok());
    assert_eq!(
        EntityUid::default().as_ref().len(),
        mem::size_of::<EntityUid>()
    );
}

#[test]
fn generate_uid() {
    assert!(EntityUid::random().validate().is_ok());
}

#[test]
fn should_encode_decode_uid() {
    let uid = EntityUid::random();
    let encoded = uid.encode_to_string();
    let decoded = EntityUid::decode_from_str(&encoded).unwrap();
    assert_eq!(uid, decoded);
}

#[test]
fn should_fail_to_decode_too_long_string() {
    let uid = EntityUid::random();
    let mut encoded = uid.encode_to_string();
    while encoded.len() <= EntityUid::MAX_STR_LEN {
        encoded.push(char::from(EntityUid::BASE58_ALPHABET[57]));
    }
    assert!(EntityUid::decode_from_str(&encoded).is_err());
}

#[test]
fn should_fail_to_decode_too_short_string() {
    let uid = EntityUid::random();
    let mut encoded = uid.encode_to_string();
    encoded.truncate(EntityUid::MIN_STR_LEN - 1);
    assert!(EntityUid::decode_from_str(&encoded).is_err());
}

#[test]
fn rev_sequence() {
    let initial = EntityRevision::initial();
    assert!(initial.validate().is_ok());
    assert!(initial.is_initial());

    let next = initial.next();
    assert!(next.validate().is_ok());
    assert!(!next.is_initial());
    assert!(initial < next);
    assert!(initial.ordinal() < next.ordinal());
    assert!(initial.instant() <= next.instant());

    let nextnext = next.next();
    assert!(nextnext.validate().is_ok());
    assert!(!nextnext.is_initial());
    assert!(next < nextnext);
    assert!(next.ordinal() < nextnext.ordinal());
    assert!(next.instant() <= nextnext.instant());
}

#[test]
fn header_without_uid() {
    let header = EntityHeader::initial_with_uid(EntityUid::default());
    assert!(!header.validate().is_ok());
    assert!(header.rev().is_initial());
}

#[test]
fn should_generate_unique_initial_headers() {
    let header1 = EntityHeader::initial();
    let header2 = EntityHeader::initial();
    assert!(header1.validate().is_ok());
    assert!(header1.rev().is_initial());
    assert!(header2.validate().is_ok());
    assert!(header2.rev().is_initial());
    assert_ne!(header1.uid(), header2.uid());
    assert_eq!(header1.rev().ordinal(), header2.rev().ordinal());
    assert!(header1.rev().instant() <= header2.rev().instant());
}
