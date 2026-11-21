// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use aoide_core_api::media::tracker::DirTrackingStatus;
use aoide_repo::RepoResult;

use crate::util::decode_repr;

pub(crate) const fn encode_dir_tracking_status(value: DirTrackingStatus) -> i16 {
    value as _
}

pub(crate) fn decode_dir_tracking_status(value: i16) -> RepoResult<DirTrackingStatus> {
    decode_repr(value, DirTrackingStatus::from_repr, "DirTrackingStatus")
}
