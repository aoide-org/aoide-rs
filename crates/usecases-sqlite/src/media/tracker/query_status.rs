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

use aoide_core_api::media::tracker::{query_status::Params, Status};

use super::*;

mod uc {
    pub(super) use aoide_usecases::media::tracker::query_status::*;
}

pub fn query_status(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Status> {
    let repo = RepoConnection::new(connection);
    uc::query_status(&repo, collection_uid, params).map_err(Into::into)
}
