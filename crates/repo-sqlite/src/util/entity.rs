// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    EncodedEntityUid, EntityHeader, EntityRevision, EntityRevisionValue, EntityUid, EntityUidTyped,
};

pub(crate) fn decode_entity_uid(uid: &str) -> EntityUid {
    EntityUid::decode_from(uid).expect("valid entity UID")
}

pub(crate) fn decode_entity_uid_typed<T: 'static>(uid: &str) -> EntityUidTyped<T> {
    EntityUidTyped::from_untyped(decode_entity_uid(uid))
}

pub(crate) fn encode_entity_uid(uid: &impl AsRef<EntityUid>) -> String {
    // TODO: Avoid dynamic allocation by using EncodedEntityUid instead of String
    let encoded = uid.as_ref().to_string();
    debug_assert_eq!(
        encoded.as_str(),
        EncodedEntityUid::from(uid.as_ref()).as_str()
    );
    encoded
}

pub(crate) fn decode_entity_revision(rev: i64) -> EntityRevision {
    let decoded = EntityRevision::new_unchecked(rev as EntityRevisionValue);
    debug_assert!(decoded.is_valid());
    decoded
}

pub(crate) const fn encode_entity_revision(rev: EntityRevision) -> i64 {
    rev.value() as _
}

pub(crate) fn decode_entity_header(uid: &str, rev: i64) -> EntityHeader {
    let uid = decode_entity_uid(uid);
    let rev = decode_entity_revision(rev);
    EntityHeader { uid, rev }
}
