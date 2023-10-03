// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use aoide_core::track::title::*;
use aoide_core_api::track::search::Scope;
use aoide_repo::track::RecordId;

use crate::prelude::*;

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
        .ok_or_else(|| anyhow::anyhow!("invalid track title Kind value: {value}").into())
}
