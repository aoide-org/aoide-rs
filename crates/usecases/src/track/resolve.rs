// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

use aoide_core::track::EntityHeader;

use aoide_repo::{collection::RecordId as CollectionId, track::CollectionRepo};

pub fn resolve_by_media_source_content_paths<Repo>(
    repo: &mut Repo,
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
