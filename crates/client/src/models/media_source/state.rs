// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::util::remote::RemoteData;

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
pub struct State {
    pub(super) remote_view: RemoteView,
}

impl State {
    #[must_use]
    pub fn remote_view(&self) -> &RemoteView {
        &self.remote_view
    }
}
