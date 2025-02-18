// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use anyhow::anyhow;

use aoide_core::track::title::*;
use aoide_core_api::track::search::Scope;
use aoide_repo::{RepoError, RepoResult, track::RecordId};

#[derive(Debug)]
pub struct Record {
    pub track_id: RecordId,
    pub scope: Scope,
    pub title: Title,
}

pub(crate) const fn encode_kind(value: Kind) -> i16 {
    value as _
}

pub(crate) fn decode_kind(value: i16) -> RepoResult<Kind> {
    u8::try_from(value)
        .ok()
        .and_then(Kind::from_repr)
        .ok_or_else(|| RepoError::Other(anyhow!("invalid track title kind value: {value}")))
}
