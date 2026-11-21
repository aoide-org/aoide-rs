// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shared logic of abortable batch tasks that operate on multiple tracks.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::anyhow;
use aoide::{
    CollectionUid,
    desktop_app::{ActionEffect, Environment},
};

use super::{Library, State};

/// The kind of a pending batch task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchTaskKind {
    ExportUnsynchronizedTracks,
    RevertUnsynchronizedTracks,
    UpdateTracks,
}

impl BatchTaskKind {
    /// Describes the pending activity for log messages.
    const fn activity(self) -> &'static str {
        match self {
            Self::ExportUnsynchronizedTracks => "exporting unsynchronized tracks",
            Self::RevertUnsynchronizedTracks => "reverting unsynchronized tracks",
            Self::UpdateTracks => "updating tracks",
        }
    }

    /// Describes the operation for log messages.
    const fn operation(self) -> &'static str {
        match self {
            Self::ExportUnsynchronizedTracks => "export of unsynchronized tracks",
            Self::RevertUnsynchronizedTracks => "revert of unsynchronized tracks",
            Self::UpdateTracks => "update of tracks",
        }
    }
}

impl Library {
    /// Check the preconditions for starting a batch task.
    ///
    /// Returns the UID of the current collection.
    pub(super) fn batch_task_collection_uid(&self) -> anyhow::Result<CollectionUid> {
        if !matches!(self.state, State::Idle) {
            return Err(anyhow!("already/still pending"));
        }
        self.shared_state
            .collection
            .read()
            .entity_uid()
            .cloned()
            .ok_or_else(|| anyhow!("no collection"))
    }

    /// Spawn a batch task and switch into the corresponding state.
    ///
    /// The preconditions must have been checked by invoking
    /// [`Self::batch_task_collection_uid()`] before.
    pub(super) fn spawn_batch_task<F, T>(
        &mut self,
        rt: &tokio::runtime::Handle,
        kind: BatchTaskKind,
        new_task: F,
    ) -> (ActionEffect, anyhow::Result<()>)
    where
        F: FnOnce(Arc<Environment>, Arc<AtomicBool>) -> T,
        T: Future<Output = ()> + Send + 'static,
    {
        debug_assert!(matches!(self.state, State::Idle));
        let abort_flag = Arc::new(AtomicBool::new(false));
        rt.spawn(new_task(Arc::clone(self.env()), Arc::clone(&abort_flag)));
        self.state = State::BatchTask { kind, abort_flag };
        (ActionEffect::Changed, Ok(()))
    }

    /// Abort the pending batch task of the given kind.
    pub(super) fn abort_batch_task(&self, kind: BatchTaskKind) -> ActionEffect {
        match &self.state {
            State::BatchTask {
                kind: pending_kind,
                abort_flag,
            } if *pending_kind == kind => {
                log::info!("Aborting {operation}", operation = kind.operation());
                abort_flag.store(true, Ordering::Relaxed);
                ActionEffect::MaybeChanged
            }
            _ => {
                log::info!("Not {activity}", activity = kind.activity());
                ActionEffect::Unchanged
            }
        }
    }

    /// Return to idle after the pending batch task of the given kind has finished.
    pub(super) fn batch_task_finished(&mut self, kind: BatchTaskKind) -> ActionEffect {
        if !self.state.is_batch_task(kind) {
            return ActionEffect::Unchanged;
        }
        self.state = State::Idle;
        ActionEffect::Changed
    }
}

impl State {
    #[must_use]
    pub(super) fn is_batch_task(&self, kind: BatchTaskKind) -> bool {
        matches!(self, Self::BatchTask { kind: pending_kind, .. } if *pending_kind == kind)
    }
}
