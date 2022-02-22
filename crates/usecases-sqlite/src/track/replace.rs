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

use aoide_core_api::track::replace::Summary;

use aoide_usecases::track::ValidatedInput;

use super::*;

mod uc {
    pub use aoide_usecases::track::replace::{
        replace_many_by_media_source_content_path, Outcome, Params,
    };
}

pub fn replace_many_by_media_source_content_path(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: &uc::Params,
    validated_track_iter: impl IntoIterator<Item = ValidatedInput>,
) -> Result<Summary> {
    let repo = RepoConnection::new(connection);
    uc::replace_many_by_media_source_content_path(
        &repo,
        collection_uid,
        params,
        validated_track_iter,
    )
    .map_err(Into::into)
}
