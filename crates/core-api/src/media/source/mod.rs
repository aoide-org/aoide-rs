// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::util::url::BaseUrl;

pub mod purge_orphaned;
pub mod purge_untracked;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ResolveUrlFromContentPath {
    #[default]
    CanonicalRootUrl,
    OverrideRootUrl {
        root_url: BaseUrl,
    },
}

impl ResolveUrlFromContentPath {
    #[must_use]
    pub const fn override_root_url(&self) -> Option<&BaseUrl> {
        match self {
            Self::CanonicalRootUrl => None,
            Self::OverrideRootUrl { root_url } => Some(root_url),
        }
    }
}
