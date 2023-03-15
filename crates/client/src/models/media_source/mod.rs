// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::EntityUid as CollectionUid;

use crate::util::remote::RemoteData;

pub mod intent;
pub use self::intent::Intent;

pub mod effect;
pub use self::effect::Effect;

pub mod task;
pub use self::task::{PendingTask, Task};

pub type Action = infect::Action<Effect, Task>;

pub type IntentHandled = infect::IntentHandled<Intent, Effect, Task>;

pub type EffectApplied = infect::EffectApplied<Effect, Task>;

#[derive(Debug, Clone)]
pub struct PurgeOrphaned {
    pub collection_uid: CollectionUid,
    pub params: aoide_core_api::media::source::purge_orphaned::Params,
}

#[derive(Debug, Clone)]
pub struct PurgeUntracked {
    pub collection_uid: CollectionUid,
    pub params: aoide_core_api::media::source::purge_untracked::Params,
}

#[derive(Debug, Default)]
pub struct RemoteView {
    pub last_purge_orphaned_outcome:
        RemoteData<aoide_core_api::media::source::purge_orphaned::Outcome>,
    pub last_purge_untracked_outcome:
        RemoteData<aoide_core_api::media::source::purge_untracked::Outcome>,
}

impl RemoteView {
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.last_purge_orphaned_outcome.is_pending()
            || self.last_purge_untracked_outcome.is_pending()
    }
}

#[derive(Debug, Default)]
pub struct Model {
    pub(super) remote_view: RemoteView,
}

impl Model {
    #[must_use]
    pub fn remote_view(&self) -> &RemoteView {
        &self.remote_view
    }
}
