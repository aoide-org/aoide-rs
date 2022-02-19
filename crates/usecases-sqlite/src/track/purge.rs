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

use aoide_repo::collection::EntityRepo as _;

use super::*;

mod uc {
    pub use aoide_usecases::{track::purge::*, Error};
}

pub fn purge_by_media_source_content_path_predicates(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    path_predicates: Vec<StringPredicate>,
) -> Result<uc::PurgeByMediaContentPathPredicatesSummary> {
    let repo = RepoConnection::new(connection);
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    uc::purge_by_media_source_content_path_predicates(&repo, collection_id, path_predicates)
        .map_err(Into::into)
}
