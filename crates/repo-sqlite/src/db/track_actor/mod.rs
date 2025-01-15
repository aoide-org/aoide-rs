// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::anyhow;

use aoide_core::track::actor::*;
use aoide_core_api::track::search::Scope;
use aoide_repo::{track::RecordId, RepoError, RepoResult};

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
    u8::try_from(value)
        .ok()
        .and_then(Role::from_repr)
        .ok_or_else(|| RepoError::Other(anyhow!("invalid track actor Role value: {value}")))
}

pub(crate) const fn encode_kind(value: Kind) -> i16 {
    value as _
}

pub(crate) fn decode_kind(value: i16) -> RepoResult<Kind> {
    u8::try_from(value)
        .ok()
        .and_then(Kind::from_repr)
        .ok_or_else(|| RepoError::Other(anyhow!("invalid track actor Kind value: {value}")))
}
