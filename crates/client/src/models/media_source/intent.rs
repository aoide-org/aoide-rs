// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{CollectionUid, Effect, IntentHandled, Model, PurgeOrphaned, PurgeUntracked};

#[derive(Debug)]
pub enum Intent {
    PurgeOrphaned {
        collection_uid: CollectionUid,
        params: aoide_core_api::media::source::purge_orphaned::Params,
    },
    PurgeUntracked {
        collection_uid: CollectionUid,
        params: aoide_core_api::media::source::purge_untracked::Params,
    },
}

impl Intent {
    #[must_use]
    pub fn handle_on(self, model: &mut Model) -> IntentHandled {
        log::trace!("Applying intent {self:?} on {model:?}");
        match self {
            Self::PurgeOrphaned {
                collection_uid,
                params,
            } => {
                if model.remote_view.last_purge_orphaned_outcome.is_pending() {
                    let self_reconstructed = Self::PurgeOrphaned {
                        collection_uid,
                        params,
                    };
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let purge_orphaned = PurgeOrphaned {
                    collection_uid,
                    params,
                };
                let effect = Effect::PurgeOrphanedAccepted(purge_orphaned);
                effect.apply_on(model).into()
            }
            Self::PurgeUntracked {
                collection_uid,
                params,
            } => {
                if model.remote_view.last_purge_untracked_outcome.is_pending() {
                    let self_reconstructed = Self::PurgeUntracked {
                        collection_uid,
                        params,
                    };
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let purge_untracked = PurgeUntracked {
                    collection_uid,
                    params,
                };
                let effect = Effect::PurgeUntrackedAccepted(purge_untracked);
                effect.apply_on(model).into()
            }
        }
    }
}
