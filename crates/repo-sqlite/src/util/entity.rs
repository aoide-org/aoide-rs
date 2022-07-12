// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::entity::*;

pub(crate) fn entity_revision_from_sql(rev: i64) -> EntityRevision {
    EntityRevision::from_inner(rev as EntityRevisionNumber)
}

pub(crate) fn entity_revision_to_sql(rev: EntityRevision) -> i64 {
    rev.to_inner() as i64
}

pub(crate) fn entity_header_from_sql(uid: &[u8], rev: i64) -> EntityHeader {
    EntityHeader {
        uid: EntityUid::from_slice(uid),
        rev: entity_revision_from_sql(rev),
    }
}
