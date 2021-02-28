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

use aoide_core::entity::EntityHeader;

use aoide_repo::{collection::RecordId as CollectionId, track::EntityRepo};

pub fn resolve_by_media_source_uris<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    media_source_uris: Vec<String>,
) -> RepoResult<Vec<(String, EntityHeader)>>
where
    Repo: EntityRepo,
{
    let mut resolved = Vec::with_capacity(media_source_uris.len());
    for media_source_uri in media_source_uris {
        let next_resolved = repo
            .resolve_track_entity_header_by_media_source_uri(collection_id, &media_source_uri)
            .optional()?;
        if let Some(next_resolved) = next_resolved {
            let (_, _, entity_header) = next_resolved;
            resolved.push((media_source_uri, entity_header));
        }
    }
    Ok(resolved)
}
