// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{media::content::ContentPath, util::url::BaseUrl};

use super::DirTrackingStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params {
    pub root_url: Option<BaseUrl>,
    pub status: Option<DirTrackingStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub root_url: BaseUrl,
    pub root_path: ContentPath<'static>,
    pub summary: Summary,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    pub untracked: usize,
}
