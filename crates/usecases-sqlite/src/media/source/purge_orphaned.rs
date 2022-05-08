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

use aoide_core_api::media::source::purge_orphaned::{Outcome, Params};

use super::*;

mod uc {
    pub(super) use aoide_usecases::media::source::purge_orphaned::purge_orphaned;
}

pub fn purge_orphaned(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Outcome> {
    let repo = RepoConnection::new(connection);
    uc::purge_orphaned(&repo, collection_uid, params).map_err(Into::into)
}
