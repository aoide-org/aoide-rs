// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::entity::EntityUid;

use crate::util::roundtrip::PendingToken;

#[derive(Debug)]
pub enum Task {
    PurgeOrphaned {
        token: PendingToken,
        collection_uid: EntityUid,
        params: aoide_core_api::media::source::purge_orphaned::Params,
    },
    PurgeUntracked {
        token: PendingToken,
        collection_uid: EntityUid,
        params: aoide_core_api::media::source::purge_untracked::Params,
    },
}

#[cfg(feature = "with-webapi-backend")]
mod webapi;