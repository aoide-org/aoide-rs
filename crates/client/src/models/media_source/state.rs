// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
