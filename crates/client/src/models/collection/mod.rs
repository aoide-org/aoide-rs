// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{CollectionEntity, CollectionUid};
use infect::ModelChanged;

use crate::util::{remote::RemoteData, roundtrip::PendingToken};

pub mod intent;
pub use self::intent::Intent;

pub mod effect;
pub use self::effect::Effect;

pub mod task;
pub use self::task::{PendingTask, Task};

pub type IntentRejected = Intent;
pub type IntentHandled = infect::IntentHandled<IntentRejected, Effect, Task, ModelChanged>;
pub type EffectApplied = infect::EffectApplied<Effect, Task, ModelChanged>;

#[derive(Debug, Clone)]
pub struct FetchFilteredEntities {
    pub filter_by_kind: Option<String>,
}

#[derive(Debug, Default)]
pub struct RemoteView {
    pub all_kinds: RemoteData<Vec<String>>,
    pub filtered_by_kind: Option<String>,
    pub filtered_entities: RemoteData<Vec<CollectionEntity>>,
}

impl RemoteView {
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.all_kinds.is_pending() || self.filtered_entities.is_pending()
    }

    pub(super) fn finish_pending_all_kinds(
        &mut self,
        token: PendingToken,
        all_kinds: Option<Vec<String>>,
    ) -> bool {
        if let Some(all_kinds) = all_kinds {
            self.all_kinds
                .finish_pending_with_value_now(token, all_kinds)
                .is_ok()
        } else {
            self.all_kinds.finish_pending(token)
        }
    }

    pub(super) fn finish_pending_filtered_entities(
        &mut self,
        token: PendingToken,
        filtered_by_kind: Option<String>,
        filtered_entities: Option<Vec<CollectionEntity>>,
    ) -> bool {
        if let Some(filtered_entities) = filtered_entities {
            if self
                .filtered_entities
                .finish_pending_with_value_now(token, filtered_entities)
                .is_ok()
            {
                self.filtered_by_kind = filtered_by_kind;
                true
            } else {
                false
            }
        } else {
            self.filtered_entities.finish_pending(token)
        }
    }

    #[must_use]
    fn count_entities_by_uid(&self, uid: &CollectionUid) -> Option<usize> {
        self.filtered_entities
            .last_value()
            .map(|v| v.iter().filter(|x| &x.hdr.uid == uid).count())
    }

    #[must_use]
    pub fn find_entity_by_uid(&self, uid: &CollectionUid) -> Option<&CollectionEntity> {
        debug_assert!(self.count_entities_by_uid(uid).unwrap_or_default() <= 1);
        self.filtered_entities
            .last_value()
            .and_then(|v| v.iter().find(|x| &x.hdr.uid == uid))
    }

    #[must_use]
    fn count_entities_by_title(&self, title: &str) -> Option<usize> {
        self.filtered_entities
            .last_value()
            .map(|v| v.iter().filter(|x| x.body.title == title).count())
    }

    #[must_use]
    pub fn find_entity_by_title(&self, title: &str) -> Option<&CollectionEntity> {
        debug_assert!(self.count_entities_by_title(title).unwrap_or_default() <= 1);
        self.filtered_entities
            .last_value()
            .and_then(|v| v.iter().find(|x| x.body.title == title))
    }
}

#[derive(Debug, Default)]
pub struct Model {
    pub(super) remote_view: RemoteView,
    pub(super) active_entity_uid: Option<CollectionUid>,
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

    #[must_use]
    pub const fn active_entity_uid(&self) -> Option<&CollectionUid> {
        self.active_entity_uid.as_ref()
    }

    #[must_use]
    pub fn active_entity(&self) -> Option<&CollectionEntity> {
        if let (Some(filtered_entities), Some(active_entity_uid)) = (
            self.remote_view.filtered_entities.last_value(),
            &self.active_entity_uid,
        ) {
            filtered_entities
                .iter()
                .find(|x| &x.hdr.uid == active_entity_uid)
        } else {
            None
        }
    }

    pub(super) fn finish_pending_all_kinds(
        &mut self,
        token: PendingToken,
        all_kinds: Option<Vec<String>>,
    ) -> bool {
        self.remote_view.finish_pending_all_kinds(token, all_kinds)
    }

    pub(super) fn finish_pending_filtered_entities(
        &mut self,
        token: PendingToken,
        filtered_by_kind: Option<String>,
        filtered_entities: Option<Vec<CollectionEntity>>,
    ) -> bool {
        let finished = self.remote_view.finish_pending_filtered_entities(
            token,
            filtered_by_kind,
            filtered_entities,
        );
        if finished {
            let active_uid = self.active_entity_uid.take();
            self.set_active_entity_uid(active_uid);
        }
        finished
    }

    #[allow(clippy::needless_pass_by_value)] // consume entity
    pub(super) fn after_entity_created_or_updated(
        &mut self,
        entity: CollectionEntity,
    ) -> EffectApplied {
        if let Some(last_snapshot) = self.remote_view.all_kinds.last_snapshot() {
            if last_snapshot
                .value
                .iter()
                .any(|kind| Some(kind) == entity.body.kind.as_ref())
            {
                // The new/modified entity is of a known kind
                return EffectApplied::unchanged();
            }
        }
        refresh_all_kinds(self)
    }

    pub(super) fn after_entity_purged(&mut self, _entity_uid: &CollectionUid) -> EffectApplied {
        refresh_all_kinds(self)
    }

    pub(super) fn set_active_entity_uid(
        &mut self,
        new_active_uid: impl Into<Option<CollectionUid>>,
    ) {
        self.active_entity_uid = if let (Some(filtered_entities), Some(new_active_uid)) = (
            self.remote_view.filtered_entities.last_value(),
            new_active_uid.into(),
        ) {
            if filtered_entities
                .iter()
                .any(|x| x.hdr.uid == new_active_uid)
            {
                Some(new_active_uid)
            } else {
                None
            }
        } else {
            None
        };
    }
}

fn refresh_all_kinds(model: &mut Model) -> EffectApplied {
    if model.remote_view().is_pending() {
        log::warn!("Cannot refresh all kinds while pending");
        return EffectApplied::unchanged();
    }
    let task = PendingTask::FetchAllKinds;
    let token = model.remote_view.all_kinds.start_pending_now();
    let task = Task::Pending { token, task };
    EffectApplied::maybe_changed_task(task)
}
