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

use crate::tag::models::*;

use aoide_core::{
    tag::*,
    track::{marker::cue::MarkerData as CueMarkerData, Entity as TrackEntity},
};

use aoide_repo::{RecordId, RepoResult};

///////////////////////////////////////////////////////////////////////
// Utility function
///////////////////////////////////////////////////////////////////////

fn cleanup_brief<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    let query = diesel::delete(
        aux_track_brief::table
            .filter(aux_track_brief::track_id.ne_all(track::table.select(track::row_id))),
    );
    query.execute(connection.as_ref())?;
    Ok(())
}

fn delete_brief<'db>(connection: &crate::Connection<'db>, record_id: RecordId) -> RepoResult<()> {
    let query =
        diesel::delete(aux_track_brief::table.filter(aux_track_brief::track_id.eq(record_id)));
    query.execute(connection.as_ref())?;
    Ok(())
}

fn insert_brief<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
    track: &Track,
) -> RepoResult<()> {
    let insertable = track::InsertableBrief::bind(record_id, track);
    let query = diesel::insert_into(aux_track_brief::table).values(&insertable);
    query.execute(connection.as_ref())?;
    Ok(())
}

fn cleanup_tags<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    // Orphaned tags
    diesel::delete(
        aux_track_tag::table
            .filter(aux_track_tag::track_id.ne_all(track::table.select(track::row_id))),
    )
    .execute(connection.as_ref())?;
    // Orphaned tag terms
    diesel::delete(
        tag_label::table.filter(
            tag_label::id
                .nullable()
                .ne_all(aux_track_tag::table.select(aux_track_tag::label_id)),
        ),
    )
    .execute(connection.as_ref())?;
    // Orphaned tag facets
    diesel::delete(
        tag_facet::table.filter(
            tag_facet::id
                .nullable()
                .ne_all(aux_track_tag::table.select(aux_track_tag::facet_id)),
        ),
    )
    .execute(connection.as_ref())?;
    Ok(())
}

fn delete_tags<'db>(connection: &crate::Connection<'db>, record_id: RecordId) -> RepoResult<()> {
    diesel::delete(aux_track_tag::table.filter(aux_track_tag::track_id.eq(record_id)))
        .execute(connection.as_ref())?;
    Ok(())
}

fn resolve_tag_label<'db>(
    connection: &crate::Connection<'db>,
    label: &Label,
) -> RepoResult<RecordId> {
    let label_str: &str = label.as_ref();
    loop {
        match tag_label::table
            .select(tag_label::id)
            .filter(tag_label::label.eq(label_str))
            .first(connection.as_ref())
            .optional()?
        {
            Some(id) => return Ok(id),
            None => {
                log::debug!("Inserting new tag label '{}'", label);
                let insertable = InsertableTagLabel::bind(label);
                diesel::insert_or_ignore_into(tag_label::table)
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
) -> RepoResult<RecordId> {
    let facet_str: &str = facet.as_ref();
    loop {
        match tag_facet::table
            .select(tag_facet::id)
            .filter(tag_facet::facet.eq(facet_str))
            .first(connection.as_ref())
            .optional()?
        {
            Some(id) => return Ok(id),
            None => {
                log::debug!("Inserting new tag facet '{}'", facet);
                let insertable = InsertableTagFacet::bind(facet);
                diesel::insert_or_ignore_into(tag_facet::table)
                    .values(&insertable)
                    .execute(connection.as_ref())?;
                // and retry to lookup the id...
            }
        }
    }
}

fn insert_tags<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
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
                InsertableTracksTag::bind(record_id, facet_id, label_id, plain_tag.score());
            match diesel::insert_into(aux_track_tag::table)
                .values(&insertable)
                .execute(connection.as_ref())
            {
                Err(err) => log::warn!(
                    "Failed to insert tag {} -> {:?} for track {}: {}",
                    facet_key,
                    plain_tag,
                    record_id,
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
            .filter(aux_track_marker::track_id.ne_all(track::table.select(track::row_id))),
    )
    .execute(connection.as_ref())?;
    // Orphaned markers labels
    diesel::delete(aux_cue_label::table.filter(
        aux_cue_label::id.ne_all(aux_track_marker::table.select(aux_track_marker::label_id)),
    ))
    .execute(connection.as_ref())?;
    Ok(())
}

fn delete_markers<'db>(connection: &crate::Connection<'db>, record_id: RecordId) -> RepoResult<()> {
    diesel::delete(aux_track_marker::table.filter(aux_track_marker::track_id.eq(record_id)))
        .execute(connection.as_ref())?;
    Ok(())
}

fn resolve_cue_label<'db>(
    connection: &crate::Connection<'db>,
    label: &str,
) -> RepoResult<RecordId> {
    loop {
        match aux_cue_label::table
            .select(aux_cue_label::id)
            .filter(aux_cue_label::label.eq(label))
            .first(connection.as_ref())
            .optional()?
        {
            Some(id) => return Ok(id),
            None => {
                log::debug!("Inserting new marker label '{}'", label);
                let insertable = InsertableCueLabel::bind(label);
                diesel::insert_or_ignore_into(aux_cue_label::table)
                    .values(&insertable)
                    .execute(connection.as_ref())?;
                // and retry to lookup the id...
            }
        }
    }
}

fn insert_markers<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
    track: &Track,
) -> RepoResult<()> {
    for marker in &track.markers.cues.markers {
        let data: &CueMarkerData = marker.data();
        if let Some(ref label) = data.label {
            let label_id = resolve_cue_label(connection, &label)?;
            let insertable = InsertableTracksMarker::bind(record_id, label_id);
            // The same label might be used for multiple markers of
            // the same track.
            match diesel::insert_or_ignore_into(aux_track_marker::table)
                .values(&insertable)
                .execute(connection.as_ref())
            {
                Err(err) => log::warn!(
                    "Failed to insert marker {:?} for track {}: {}",
                    marker,
                    record_id,
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
    Ok(())
}

fn after_insert<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
    track: &Track,
) -> RepoResult<()> {
    insert_brief(connection, record_id, track)?;
    insert_markers(connection, record_id, track)?;
    insert_tags(connection, record_id, track)?;
    Ok(())
}

pub fn before_delete_or_update<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
) -> RepoResult<()> {
    delete_tags(connection, record_id)?;
    delete_markers(connection, record_id)?;
    delete_brief(connection, record_id)?;
    Ok(())
}

fn on_refresh<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
    track: &Track,
) -> RepoResult<()> {
    before_delete_or_update(connection, record_id)?;
    after_insert(connection, record_id, track)?;
    Ok(())
}

pub fn refresh_entity<'db>(
    connection: &crate::Connection<'db>,
    entity: &TrackEntity,
) -> RepoResult<RecordId> {
    let uid = &entity.hdr.uid;
    match connection.resolve_track_id(uid)? {
        Some(record_id) => {
            on_refresh(connection, record_id, &entity.body)?;
            Ok(record_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn after_entity_inserted<'db>(
    connection: &crate::Connection<'db>,
    track: &Track,
    hdr: &EntityHeader,
) -> RepoResult<RecordId> {
    let uid = &hdr.uid;
    match connection.resolve_track_id(uid)? {
        Some(record_id) => {
            after_insert(connection, record_id, &track)?;
            Ok(record_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn before_entity_updated_or_removed<'db>(
    connection: &crate::Connection<'db>,
    uid: &EntityUid,
) -> RepoResult<RecordId> {
    match connection.resolve_track_id(uid)? {
        Some(record_id) => {
            before_delete_or_update(connection, record_id)?;
            Ok(record_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn after_entity_updated<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
    track: &Track,
) -> RepoResult<()> {
    after_insert(connection, record_id, track)?;
    Ok(())
}
