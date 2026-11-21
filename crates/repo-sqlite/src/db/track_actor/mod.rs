// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::track::actor::*;
use aoide_core_api::track::search::Scope;
use aoide_repo::{RepoResult, track::RecordId};

use crate::util::decode_repr;

pub(crate) mod models;
pub(crate) mod schema;

#[derive(Debug)]
pub struct Record {
    pub track_id: RecordId,
    pub scope: Scope,
    pub actor: Actor,
}

pub(crate) const fn encode_role(value: Role) -> i16 {
    value as _
}

pub(crate) fn decode_role(value: i16) -> RepoResult<Role> {
    decode_repr(value, Role::from_repr, "track actor Role")
}

pub(crate) const fn encode_kind(value: Kind) -> i16 {
    value as _
}

pub(crate) fn decode_kind(value: i16) -> RepoResult<Kind> {
    decode_repr(value, Kind::from_repr, "track actor Kind")
}
