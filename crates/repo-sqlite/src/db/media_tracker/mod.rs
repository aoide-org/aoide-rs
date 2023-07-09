// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use aoide_core_api::media::tracker::DirTrackingStatus;

use crate::prelude::*;

pub(crate) fn encode_dir_tracking_status(value: DirTrackingStatus) -> i16 {
    value as _
}

pub(crate) fn decode_dir_tracking_status(value: i16) -> RepoResult<DirTrackingStatus> {
    u8::try_from(value)
        .ok()
        .and_then(DirTrackingStatus::from_repr)
        .ok_or_else(|| anyhow::anyhow!("invalid DirTrackingStatus value: {value}").into())
}
