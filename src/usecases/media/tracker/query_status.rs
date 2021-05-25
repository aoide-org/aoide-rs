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

use aoide_core::entity::EntityUid;

use url::Url;

///////////////////////////////////////////////////////////////////////

pub use aoide_core::media::tracker::Status;

pub fn query_status(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    root_dir_url: &Url,
) -> Result<Status> {
    let path_prefix = path_prefix_from_url(root_dir_url)?;
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselTransactionError<RepoError>, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let directories =
            db.media_tracker_aggregate_directories_tracking_status(collection_id, &path_prefix)?;
        let status = Status { directories };
        Ok(status)
    })
    .map_err(Into::into)
}
