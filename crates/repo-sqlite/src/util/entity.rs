// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::entity::{
    EncodedEntityUid, EntityHeader, EntityRevision, EntityRevisionNumber, EntityUid, EntityUidTyped,
};

pub(crate) fn entity_uid_from_sql(uid: &str) -> EntityUid {
    EntityUid::decode_from(uid).unwrap()
}

pub(crate) fn entity_uid_typed_from_sql<T: 'static>(uid: &str) -> EntityUidTyped<T> {
    EntityUidTyped::from_untyped(entity_uid_from_sql(uid))
}

pub(crate) fn entity_uid_to_sql(uid: &impl AsRef<EntityUid>) -> String {
    // TODO: Avoid dynamic allocation by using EncodedEntityUid instead of String
    let encoded = uid.as_ref().to_string();
    debug_assert_eq!(
        encoded.as_str(),
        EncodedEntityUid::from(uid.as_ref()).as_str()
    );
    encoded
}

pub(crate) fn entity_revision_from_sql(rev: i64) -> EntityRevision {
    EntityRevision::from_inner(rev as EntityRevisionNumber)
}

pub(crate) fn entity_revision_to_sql(rev: EntityRevision) -> i64 {
    rev.to_inner() as i64
}

pub(crate) fn entity_header_from_sql(uid: &str, rev: i64) -> EntityHeader {
    let uid = entity_uid_from_sql(uid);
    let rev = entity_revision_from_sql(rev);
    EntityHeader { uid, rev }
}
