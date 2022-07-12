// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{media::content::ContentPath, util::url::BaseUrl};

use super::Completion;

pub type Params = super::FsTraversalParams;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub root_url: BaseUrl,
    pub root_path: ContentPath,
    pub completion: Completion,
    pub content_paths: Vec<ContentPath>,
}
