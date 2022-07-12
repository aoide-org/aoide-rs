// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{media::content::ContentPath, util::url::BaseUrl};

use super::{Completion, FsTraversalParams};

pub type Params = FsTraversalParams;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    pub current: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub root_url: BaseUrl,
    pub root_path: ContentPath,
    pub completion: Completion,
    pub summary: Summary,
}
