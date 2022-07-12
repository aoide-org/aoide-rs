// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{media::content::ContentPath, util::url::BaseUrl};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Params {
    pub root_url: Option<BaseUrl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub root_url: Option<BaseUrl>,
    pub root_path: Option<ContentPath>,
    pub summary: Summary,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    pub purged: usize,
}
