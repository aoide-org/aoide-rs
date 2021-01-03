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

use aoide_core::{
    tag::*,
    track::{marker::cue::MarkerData as CueMarkerData, Entity as TrackEntity},
};

use aoide_repo::{RepoId, RepoResult};

///////////////////////////////////////////////////////////////////////
// Utility function
///////////////////////////////////////////////////////////////////////

fn cleanup_media<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    let query = diesel::delete(
        aux_track_media::table
            .filter(aux_track_media::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
    );
    query.execute(connection.as_ref())?;
    Ok(())
}

fn delete_media<'db>(connection: &crate::Connection<'db>, repo_id: RepoId) -> RepoResult<()> {
    let query =
        diesel::delete(aux_track_media::table.filter(aux_track_media::track_id.eq(repo_id)));
    query.execute(connection.as_ref())?;
    Ok(())
}

fn insert_media<'db>(
    connection: &crate::Connection<'db>,
    repo_id: RepoId,
    track: &Track,
) -> RepoResult<()> {
    for media_source in &track.media_sources {
        let insertable = track::InsertableSource::bind(repo_id, media_source);
        let query = diesel::insert_into(aux_track_media::table).values(&insertable);
        query.execute(connection.as_ref())?;
    }
    Ok(())
}

fn cleanup_location<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    let query = diesel::delete(
        aux_track_location::table
            .filter(aux_track_location::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
    );
    query.execute(connection.as_ref())?;
    Ok(())
}

fn delete_location<'db>(connection: &crate::Connection<'db>, repo_id: RepoId) -> RepoResult<()> {
    let query =
        diesel::delete(aux_track_location::table.filter(aux_track_location::track_id.eq(repo_id)));
    query.execute(connection.as_ref())?;
    Ok(())
}

fn insert_location<'db>(
    connection: &crate::Connection<'db>,
    collection_uid: &EntityUid,
    repo_id: RepoId,
    track: &Track,
) -> RepoResult<()> {
    for media_source in &track.media_sources {
        let insertable =
            track::InsertableLocation::bind(repo_id, collection_uid.as_ref(), &media_source.uri);
        let query = diesel::insert_into(aux_track_location::table).values(&insertable);
        query.execute(connection.as_ref())?;
    }
    Ok(())
}

fn cleanup_brief<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    let query = diesel::delete(
        aux_track_brief::table
            .filter(aux_track_brief::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
    );
    query.execute(connection.as_ref())?;
    Ok(())
}

fn delete_brief<'db>(connection: &crate::Connection<'db>, repo_id: RepoId) -> RepoResult<()> {
    let query =
        diesel::delete(aux_track_brief::table.filter(aux_track_brief::track_id.eq(repo_id)));
    query.execute(connection.as_ref())?;
    Ok(())
}

fn insert_brief<'db>(
    connection: &crate::Connection<'db>,
    repo_id: RepoId,
    track: &Track,
) -> RepoResult<()> {
    let insertable = track::InsertableBrief::bind(repo_id, track);
    let query = diesel::insert_into(aux_track_brief::table).values(&insertable);
    query.execute(connection.as_ref())?;
    Ok(())
}

fn cleanup_tags<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    // Orphaned tags
    diesel::delete(
        aux_track_tag::table
            .filter(aux_track_tag::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
    )
    .execute(connection.as_ref())?;
    // Orphaned tag terms
    diesel::delete(
        aux_tag_label::table.filter(
            aux_tag_label::id
                .nullable()
                .ne_all(aux_track_tag::table.select(aux_track_tag::label_id)),
        ),
    )
    .execute(connection.as_ref())?;
    // Orphaned tag facets
    diesel::delete(
        aux_tag_facet::table.filter(
            aux_tag_facet::id
                .nullable()
                .ne_all(aux_track_tag::table.select(aux_track_tag::facet_id)),
        ),
    )
    .execute(connection.as_ref())?;
    Ok(())
}

fn delete_tags<'db>(connection: &crate::Connection<'db>, repo_id: RepoId) -> RepoResult<()> {
    diesel::delete(aux_track_tag::table.filter(aux_track_tag::track_id.eq(repo_id)))
        .execute(connection.as_ref())?;
    Ok(())
}

fn resolve_tag_label<'db>(
    connection: &crate::Connection<'db>,
    label: &Label,
) -> RepoResult<RepoId> {
    let label_str: &str = label.as_ref();
    loop {
        match aux_tag_label::table
            .select(aux_tag_label::id)
            .filter(aux_tag_label::label.eq(label_str))
            .first(connection.as_ref())
            .optional()?
        {
            Some(id) => return Ok(id),
            None => {
                log::debug!("Inserting new tag label '{}'", label);
                let insertable = InsertableTagLabel::bind(label);
                diesel::insert_or_ignore_into(aux_tag_label::table)
                    .values(&insertable)
                    .execute(connection.as_ref())?;
                // and retry to lookup the id...
            }
        }
    }
}

fn resolve_tag_facet<'db>(
    connection: &crate::Connection<'db>,
    facet: &Facet,
) -> RepoResult<RepoId> {
    let facet_str: &str = facet.as_ref();
    loop {
        match aux_tag_facet::table
            .select(aux_tag_facet::id)
            .filter(aux_tag_facet::facet.eq(facet_str))
            .first(connection.as_ref())
            .optional()?
        {
            Some(id) => return Ok(id),
            None => {
                log::debug!("Inserting new tag facet '{}'", facet);
                let insertable = InsertableTagFacet::bind(facet);
                diesel::insert_or_ignore_into(aux_tag_facet::table)
                    .values(&insertable)
                    .execute(connection.as_ref())?;
                // and retry to lookup the id...
            }
        }
    }
}

fn insert_tags<'db>(
    connection: &crate::Connection<'db>,
    repo_id: RepoId,
    track: &Track,
) -> RepoResult<()> {
    for (facet_key, plain_tags) in track.tags.as_ref() {
        let facet: &Option<Facet> = facet_key.as_ref();
        let facet_id = facet
            .as_ref()
            .map(|facet| resolve_tag_facet(connection, facet))
            .transpose()?;
        for plain_tag in plain_tags {
            let label_id = plain_tag
                .label
                .as_ref()
                .map(|label| resolve_tag_label(connection, label))
                .transpose()?;
            let insertable =
                InsertableTracksTag::bind(repo_id, facet_id, label_id, plain_tag.score());
            match diesel::insert_into(aux_track_tag::table)
                .values(&insertable)
                .execute(connection.as_ref())
            {
                Err(err) => log::warn!(
                    "Failed to insert tag {} -> {:?} for track {}: {}",
                    facet_key,
                    plain_tag,
                    repo_id,
                    err
                ),
                Ok(count) => debug_assert!(count == 1),
            }
        }
    }
    Ok(())
}

fn cleanup_markers<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    // Orphaned markers
    diesel::delete(
        aux_track_marker::table
            .filter(aux_track_marker::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
    )
    .execute(connection.as_ref())?;
    // Orphaned markers labels
    diesel::delete(aux_marker_label::table.filter(
        aux_marker_label::id.ne_all(aux_track_marker::table.select(aux_track_marker::label_id)),
    ))
    .execute(connection.as_ref())?;
    Ok(())
}

fn delete_markers<'db>(connection: &crate::Connection<'db>, repo_id: RepoId) -> RepoResult<()> {
    diesel::delete(aux_track_marker::table.filter(aux_track_marker::track_id.eq(repo_id)))
        .execute(connection.as_ref())?;
    Ok(())
}

fn resolve_marker_label<'db>(
    connection: &crate::Connection<'db>,
    label: &str,
) -> RepoResult<RepoId> {
    loop {
        match aux_marker_label::table
            .select(aux_marker_label::id)
            .filter(aux_marker_label::label.eq(label))
            .first(connection.as_ref())
            .optional()?
        {
            Some(id) => return Ok(id),
            None => {
                log::debug!("Inserting new marker label '{}'", label);
                let insertable = InsertableMarkerLabel::bind(label);
                diesel::insert_or_ignore_into(aux_marker_label::table)
                    .values(&insertable)
                    .execute(connection.as_ref())?;
                // and retry to lookup the id...
            }
        }
    }
}

fn insert_markers<'db>(
    connection: &crate::Connection<'db>,
    repo_id: RepoId,
    track: &Track,
) -> RepoResult<()> {
    for marker in &track.markers.cues.markers {
        let data: &CueMarkerData = marker.data();
        if let Some(ref label) = data.label {
            let label_id = resolve_marker_label(connection, &label)?;
            let insertable = InsertableTracksMarker::bind(repo_id, label_id);
            // The same label might be used for multiple markers of
            // the same track.
            match diesel::insert_or_ignore_into(aux_track_marker::table)
                .values(&insertable)
                .execute(connection.as_ref())
            {
                Err(err) => log::warn!(
                    "Failed to insert marker {:?} for track {}: {}",
                    marker,
                    repo_id,
                    err
                ),
                Ok(count) => debug_assert!(count <= 1),
            }
        }
    }
    Ok(())
}

pub fn cleanup<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    cleanup_tags(connection)?;
    cleanup_markers(connection)?;
    cleanup_brief(connection)?;
    cleanup_location(connection)?;
    cleanup_media(connection)?;
    Ok(())
}

fn on_insert<'db>(
    connection: &crate::Connection<'db>,
    collection_uid: Option<&EntityUid>,
    repo_id: RepoId,
    track: &Track,
) -> RepoResult<()> {
    insert_media(connection, repo_id, track)?;
    if let Some(collection_uid) = collection_uid {
        insert_location(connection, collection_uid, repo_id, track)?;
    }
    insert_brief(connection, repo_id, track)?;
    insert_markers(connection, repo_id, track)?;
    insert_tags(connection, repo_id, track)?;
    Ok(())
}

fn on_delete<'db>(connection: &crate::Connection<'db>, repo_id: RepoId) -> RepoResult<()> {
    delete_tags(connection, repo_id)?;
    delete_markers(connection, repo_id)?;
    delete_brief(connection, repo_id)?;
    delete_location(connection, repo_id)?;
    delete_media(connection, repo_id)?;
    Ok(())
}

fn on_refresh<'db>(
    connection: &crate::Connection<'db>,
    collection_uid: Option<&EntityUid>,
    repo_id: RepoId,
    track: &Track,
) -> RepoResult<()> {
    on_delete(connection, repo_id)?;
    on_insert(connection, collection_uid, repo_id, track)?;
    Ok(())
}

pub fn refresh_entity<'db>(
    connection: &crate::Connection<'db>,
    collection_uid: Option<&EntityUid>,
    entity: &TrackEntity,
) -> RepoResult<RepoId> {
    let uid = &entity.hdr.uid;
    match connection.resolve_track_id(uid)? {
        Some(repo_id) => {
            on_refresh(connection, collection_uid, repo_id, &entity.body)?;
            Ok(repo_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn after_entity_inserted<'db>(
    connection: &crate::Connection<'db>,
    collection_uid: Option<&EntityUid>,
    entity: &TrackEntity,
) -> RepoResult<RepoId> {
    let uid = &entity.hdr.uid;
    match connection.resolve_track_id(uid)? {
        Some(repo_id) => {
            on_insert(connection, collection_uid, repo_id, &entity.body)?;
            Ok(repo_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn before_entity_updated_or_removed<'db>(
    connection: &crate::Connection<'db>,
    uid: &EntityUid,
) -> RepoResult<RepoId> {
    match connection.resolve_track_id(uid)? {
        Some(repo_id) => {
            on_delete(connection, repo_id)?;
            Ok(repo_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn after_entity_updated<'db>(
    connection: &crate::Connection<'db>,
    collection_uid: Option<&EntityUid>,
    repo_id: RepoId,
    track: &Track,
) -> RepoResult<()> {
    on_insert(connection, collection_uid, repo_id, track)?;
    Ok(())
}
