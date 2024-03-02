// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::AddAssign;

use aoide_core::{media::content::ContentPath, util::url::BaseUrl};

use super::Completion;
use crate::{media::SyncMode, track::replace::Summary as TrackReplaceSummary};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params {
    pub root_url: Option<BaseUrl>,
    pub sync_mode: SyncMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedSourceWithIssues {
    pub path: ContentPath<'static>,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub root_url: BaseUrl,
    pub root_path: ContentPath<'static>,
    pub completion: Completion,
    pub summary: Summary,
    pub imported_sources_with_issues: Vec<ImportedSourceWithIssues>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Summary {
    pub tracks: TrackSummary,
    pub directories: DirectorySummary,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TrackSummary {
    pub created: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub skipped: usize,
    pub failed: usize,
    pub not_imported: usize,
    pub not_created: usize,
    pub not_updated: usize,
}

impl AddAssign<&TrackReplaceSummary> for TrackSummary {
    fn add_assign(&mut self, rhs: &TrackReplaceSummary) {
        let Self {
            created,
            updated,
            unchanged,
            skipped,
            failed,
            not_imported,
            not_created,
            not_updated,
        } = self;
        let TrackReplaceSummary {
            created: rhs_created,
            updated: rhs_updated,
            unchanged: rhs_unchanged,
            skipped: rhs_skipped,
            failed: rhs_failed,
            not_imported: rhs_not_imported,
            not_created: rhs_not_created,
            not_updated: rhs_not_updated,
        } = rhs;
        *created += rhs_created.len();
        *updated += rhs_updated.len();
        *unchanged += rhs_unchanged.len();
        *skipped += rhs_skipped.len();
        *failed += rhs_failed.len();
        *not_imported += rhs_not_imported.len();
        *not_created += rhs_not_created.len();
        *not_updated += rhs_not_updated.len();
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DirectorySummary {
    pub confirmed: usize,
    pub skipped: usize,
    pub untracked: usize,
}
