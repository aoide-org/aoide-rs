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

use super::*;

use aoide_repo::{collection::EntityRepo as _, media::source::Repo as _};

///////////////////////////////////////////////////////////////////////

pub fn purge_by_media_source_uri_predicates(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    uri_predicates: Vec<StringPredicate>,
) -> Result<usize> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselRepoError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let mut total_purged_tracks = 0;
        for uri_predicate in uri_predicates {
            let purged_tracks = db.purge_tracks_by_media_source_uri_predicate(
                collection_id,
                uri_predicate.borrow(),
            )?;
            let _purged_media_sources =
                db.purge_media_sources_by_uri_predicate(collection_id, uri_predicate.borrow())?;
            debug_assert_eq!(purged_tracks, _purged_media_sources);
            total_purged_tracks += purged_tracks;
        }
        Ok(total_purged_tracks)
    })
    .map_err(Into::into)
}
