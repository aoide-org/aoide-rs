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

use aoide_repo::{
    collection::EntityRepo as _,
    track::{ReplaceMode, ReplaceOutcome},
};

///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub created: Vec<Entity>,
    pub updated: Vec<Entity>,
    pub unchanged: Vec<Entity>,
    pub not_created: Vec<Track>,
    pub not_updated: Vec<Track>,
}

pub fn replace_by_media_source_uri(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    replace_mode: ReplaceMode,
    tracks: impl Iterator<Item = Track>,
) -> RepoResult<Outcome> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselRepoError, _>(|| {
        let mut outcome = Outcome::default();
        let collection_id = db.resolve_collection_id(collection_uid)?;
        for track in tracks {
            let uri = track.media_source.uri.clone();
            let replace_outcome = db
                .replace_collected_track_by_media_source_uri(collection_id, replace_mode, track)
                .map_err(|err| {
                    log::warn!("Failed to replace track by URI {}: {}", uri, err);
                    err
                })?;
            match replace_outcome {
                ReplaceOutcome::Created(_, entity) => {
                    debug_assert_ne!(ReplaceMode::UpdateOnly, replace_mode);
                    log::debug!("Created {}: {:?}", entity.body.media_source.uri, entity.hdr);
                    outcome.created.push(entity);
                }
                ReplaceOutcome::Updated(_, entity) => {
                    debug_assert_ne!(ReplaceMode::CreateOnly, replace_mode);
                    log::debug!("Updated {}: {:?}", entity.body.media_source.uri, entity.hdr);
                    outcome.updated.push(entity);
                }
                ReplaceOutcome::Unchanged(_, entity) => {
                    log::debug!("Unchanged: {:?}", entity);
                    outcome.unchanged.push(entity);
                }
                ReplaceOutcome::NotCreated(track) => {
                    debug_assert_eq!(ReplaceMode::UpdateOnly, replace_mode);
                    log::debug!("Not created: {:?}", track);
                    outcome.not_created.push(track);
                }
                ReplaceOutcome::NotUpdated(_, track) => {
                    debug_assert_eq!(ReplaceMode::CreateOnly, replace_mode);
                    log::debug!("Not updated: {:?}", track);
                    outcome.not_created.push(track);
                }
            }
        }
        Ok(outcome)
    })
    .map_err(Into::into)
}
