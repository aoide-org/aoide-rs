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

use aoide_core_api::track::find_unsynchronized::{Params, UnsynchronizedTrackEntity};

use aoide_usecases::track::find_unsynchronized as uc;

use super::*;

pub fn find_unsynchronized(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    params: Params,
    pagination: &Pagination,
) -> Result<Vec<UnsynchronizedTrackEntity>> {
    let repo = RepoConnection::new(connection);
    uc::find_unsynchronized_with_params(&repo, collection_uid, params, pagination)
        .map_err(Into::into)
}
