// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::CollectionUid;
use infect::ModelChanged;

use crate::util::remote::RemoteData;

pub mod effect;

pub use self::effect::Effect;

pub mod intent;
pub use self::intent::Intent;

pub mod task;
pub use self::task::{PendingTask, Task};

pub type IntentRejected = Intent;
pub type IntentHandled = infect::IntentHandled<IntentRejected, Effect, Task, ModelChanged>;
pub type EffectApplied = infect::EffectApplied<Effect, Task, ModelChanged>;

#[derive(Debug, Clone)]
pub struct FetchStatus {
    pub collection_uid: CollectionUid,
    pub params: aoide_core_api::media::tracker::query_status::Params,
}

#[derive(Debug, Clone)]
pub struct StartScanDirectories {
    pub collection_uid: CollectionUid,
    pub params: aoide_core_api::media::tracker::scan_directories::Params,
}

#[derive(Debug, Clone)]
pub struct StartImportFiles {
    pub collection_uid: CollectionUid,
    pub params: aoide_core_api::media::tracker::import_files::Params,
}

#[derive(Debug, Clone)]
pub struct StartFindUntrackedFiles {
    pub collection_uid: CollectionUid,
    pub params: aoide_core_api::media::tracker::find_untracked_files::Params,
}

#[derive(Debug, Clone)]
pub struct UntrackDirectories {
    pub collection_uid: CollectionUid,
    pub params: aoide_core_api::media::tracker::untrack_directories::Params,
}

#[derive(Debug, Default)]
pub struct RemoteView {
    pub status: RemoteData<aoide_core_api::media::tracker::Status>,
    pub progress: RemoteData<aoide_core_api::media::tracker::Progress>,
    pub last_scan_directories_outcome:
        RemoteData<aoide_core_api::media::tracker::scan_directories::Outcome>,
    pub last_untrack_directories_outcome:
        RemoteData<aoide_core_api::media::tracker::untrack_directories::Outcome>,
    pub last_import_files_outcome:
        RemoteData<aoide_core_api::media::tracker::import_files::Outcome>,
    pub last_find_untracked_files_outcome:
        RemoteData<aoide_core_api::media::tracker::find_untracked_files::Outcome>,
}

impl RemoteView {
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.status.is_pending()
            || self.progress.is_pending()
            || self.last_scan_directories_outcome.is_pending()
            || self.last_untrack_directories_outcome.is_pending()
            || self.last_import_files_outcome.is_pending()
            || self.last_find_untracked_files_outcome.is_pending()
    }
}

#[derive(Debug, Default)]
pub struct Model {
    pub(super) remote_view: RemoteView,
    pub(super) last_error: Option<anyhow::Error>,
}

impl Model {
    #[must_use]
    pub const fn remote_view(&self) -> &RemoteView {
        &self.remote_view
    }

    #[must_use]
    pub const fn last_error(&self) -> Option<&anyhow::Error> {
        self.last_error.as_ref()
    }
}
