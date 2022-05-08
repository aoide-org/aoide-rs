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

use super::*;

use aoide_core::track::EntityHeader;

use aoide_repo::{collection::RecordId as CollectionId, track::CollectionRepo};

pub fn resolve_by_media_source_content_paths<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    content_paths: Vec<String>,
) -> RepoResult<Vec<(String, EntityHeader)>>
where
    Repo: CollectionRepo,
{
    let mut resolved = Vec::with_capacity(content_paths.len());
    for content_path in content_paths {
        let next_resolved = repo
            .resolve_track_entity_header_by_media_source_content_path(collection_id, &content_path)
            .optional()?;
        if let Some(next_resolved) = next_resolved {
            let (_, _, entity_header) = next_resolved;
            resolved.push((content_path, entity_header));
        }
    }
    Ok(resolved)
}
