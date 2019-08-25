// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    collection::{Collection, Entity as CollectionEntity},
    tag::*,
    track::{marker::position::MarkerData as PositionMarkerData, Entity as TrackEntity},
};

use aoide_repo::{
    collection::Repo as CollectionRepo, entity::Repo as EntityRepo, RepoId, RepoResult,
};

use crate::collection::{schema::tbl_collection, Repository as CollectionRepository};

use failure::Error;

///////////////////////////////////////////////////////////////////////
// RepositoryHelper
///////////////////////////////////////////////////////////////////////

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct RepositoryHelper<'a> {
    connection: &'a diesel::SqliteConnection,
}

impl<'a> RepositoryHelper<'a> {
    pub fn new(connection: &'a diesel::SqliteConnection) -> Self {
        Self { connection }
    }

    pub fn recreate_missing_collections(
        &self,
        collection_prototype: &Collection,
    ) -> Result<Vec<CollectionEntity>, Error> {
        let orphaned_collection_uids = aux_track_collection::table
            .select(aux_track_collection::collection_uid)
            .distinct()
            .filter(
                aux_track_collection::collection_uid
                    .ne_all(tbl_collection::table.select(tbl_collection::uid)),
            )
            .load::<Vec<u8>>(self.connection)?;
        let mut recreated_collections = Vec::with_capacity(orphaned_collection_uids.len());
        if !orphaned_collection_uids.is_empty() {
            let collection_repo = CollectionRepository::new(self.connection);
            for collection_uid in orphaned_collection_uids {
                let uid = EntityUid::from_slice(&collection_uid);
                log::info!("Recreating missing collection '{}'", uid.to_string());
                let collection_entity = CollectionEntity::new(
                    EntityHeader::initial_with_uid(uid),
                    collection_prototype.clone(),
                );
                collection_repo.insert_collection(&collection_entity)?;
                recreated_collections.push(collection_entity);
            }
        }
        Ok(recreated_collections)
    }

    fn cleanup_source(&self) -> RepoResult<()> {
        let query = diesel::delete(
            aux_track_source::table
                .filter(aux_track_source::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_source(&self, track_id: RepoId) -> RepoResult<()> {
        let query =
            diesel::delete(aux_track_source::table.filter(aux_track_source::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_source(&self, track_id: RepoId, track: &Track) -> RepoResult<()> {
        for media_source in &track.media_sources {
            let insertable = track::InsertableSource::bind(track_id, media_source);
            let query = diesel::insert_into(aux_track_source::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_collection(&self) -> RepoResult<()> {
        let query =
            diesel::delete(aux_track_collection::table.filter(
                aux_track_collection::track_id.ne_all(tbl_track::table.select(tbl_track::id)),
            ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_collection(&self, track_id: RepoId) -> RepoResult<()> {
        let query = diesel::delete(
            aux_track_collection::table.filter(aux_track_collection::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_collection(&self, track_id: RepoId, track: &Track) -> RepoResult<()> {
        for collection in &track.collections {
            let insertable = track::InsertableCollection::bind(track_id, collection);
            let query = diesel::insert_into(aux_track_collection::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_brief(&self) -> RepoResult<()> {
        let query = diesel::delete(
            aux_track_brief::table
                .filter(aux_track_brief::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_brief(&self, track_id: RepoId) -> RepoResult<()> {
        let query =
            diesel::delete(aux_track_brief::table.filter(aux_track_brief::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_brief(&self, track_id: RepoId, track: &Track) -> RepoResult<()> {
        let insertable = track::InsertableBrief::bind(track_id, track);
        let query = diesel::insert_into(aux_track_brief::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn cleanup_tags(&self) -> RepoResult<()> {
        // Orphaned tags
        diesel::delete(
            aux_track_tag::table
                .filter(aux_track_tag::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        )
        .execute(self.connection)?;
        // Orphaned tag terms
        diesel::delete(
            aux_tag_label::table.filter(
                aux_tag_label::id
                    .nullable()
                    .ne_all(aux_track_tag::table.select(aux_track_tag::label_id)),
            ),
        )
        .execute(self.connection)?;
        // Orphaned tag facets
        diesel::delete(
            aux_tag_facet::table.filter(
                aux_tag_facet::id
                    .nullable()
                    .ne_all(aux_track_tag::table.select(aux_track_tag::facet_id)),
            ),
        )
        .execute(self.connection)?;
        Ok(())
    }

    fn delete_tags(&self, track_id: RepoId) -> RepoResult<()> {
        diesel::delete(aux_track_tag::table.filter(aux_track_tag::track_id.eq(track_id)))
            .execute(self.connection)?;
        Ok(())
    }

    fn resolve_tag_label(&self, label: &Label) -> RepoResult<RepoId> {
        let label_str: &str = label.as_ref();
        loop {
            match aux_tag_label::table
                .select(aux_tag_label::id)
                .filter(aux_tag_label::label.eq(label_str))
                .first(self.connection)
                .optional()?
            {
                Some(id) => return Ok(id),
                None => {
                    log::debug!("Inserting new tag label '{}'", label);
                    let insertable = InsertableTagLabel::bind(label);
                    diesel::insert_or_ignore_into(aux_tag_label::table)
                        .values(&insertable)
                        .execute(self.connection)?;
                    // and retry to lookup the id...
                }
            }
        }
    }

    fn resolve_tag_facet(&self, facet: &Facet) -> RepoResult<RepoId> {
        let facet_str: &str = facet.as_ref();
        loop {
            match aux_tag_facet::table
                .select(aux_tag_facet::id)
                .filter(aux_tag_facet::facet.eq(facet_str))
                .first(self.connection)
                .optional()?
            {
                Some(id) => return Ok(id),
                None => {
                    log::debug!("Inserting new tag facet '{}'", facet);
                    let insertable = InsertableTagFacet::bind(facet);
                    diesel::insert_or_ignore_into(aux_tag_facet::table)
                        .values(&insertable)
                        .execute(self.connection)?;
                    // and retry to lookup the id...
                }
            }
        }
    }

    fn insert_tags(&self, track_id: RepoId, track: &Track) -> RepoResult<()> {
        for tag in &track.tags {
            let facet_id = tag
                .facet()
                .map(|facet| self.resolve_tag_facet(facet))
                .transpose()?;
            let label_id = tag
                .label()
                .map(|label| self.resolve_tag_label(label))
                .transpose()?;
            let insertable = InsertableTracksTag::bind(track_id, facet_id, label_id, tag.score());
            match diesel::insert_into(aux_track_tag::table)
                .values(&insertable)
                .execute(self.connection)
            {
                Err(err) => log::error!(
                    "Failed to insert tag {:?} for track {}: {}",
                    tag,
                    track_id,
                    err
                ),
                Ok(count) => debug_assert!(count == 1),
            }
        }
        Ok(())
    }

    fn cleanup_markers(&self) -> RepoResult<()> {
        // Orphaned markers
        diesel::delete(
            aux_track_marker::table
                .filter(aux_track_marker::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        )
        .execute(self.connection)?;
        // Orphaned markers labels
        diesel::delete(aux_marker_label::table.filter(
            aux_marker_label::id.ne_all(aux_track_marker::table.select(aux_track_marker::label_id)),
        ))
        .execute(self.connection)?;
        Ok(())
    }

    fn delete_markers(&self, track_id: RepoId) -> RepoResult<()> {
        diesel::delete(aux_track_marker::table.filter(aux_track_marker::track_id.eq(track_id)))
            .execute(self.connection)?;
        Ok(())
    }

    fn resolve_marker_label(&self, label: &str) -> RepoResult<RepoId> {
        loop {
            match aux_marker_label::table
                .select(aux_marker_label::id)
                .filter(aux_marker_label::label.eq(label))
                .first(self.connection)
                .optional()?
            {
                Some(id) => return Ok(id),
                None => {
                    log::debug!("Inserting new marker label '{}'", label);
                    let insertable = InsertableMarkerLabel::bind(label);
                    diesel::insert_or_ignore_into(aux_marker_label::table)
                        .values(&insertable)
                        .execute(self.connection)?;
                    // and retry to lookup the id...
                }
            }
        }
    }

    fn insert_markers(&self, track_id: RepoId, track: &Track) -> RepoResult<()> {
        for marker in &track.markers.positions {
            let data: &PositionMarkerData = marker.data();
            if let Some(ref label) = data.label {
                let label_id = self.resolve_marker_label(&label)?;
                let insertable = InsertableTracksMarker::bind(track_id, label_id);
                // The same label might be used for multiple markers of
                // the same track.
                match diesel::insert_or_ignore_into(aux_track_marker::table)
                    .values(&insertable)
                    .execute(self.connection)
                {
                    Err(err) => log::error!(
                        "Failed to insert marker {:?} for track {}: {}",
                        marker,
                        track_id,
                        err
                    ),
                    Ok(count) => debug_assert!(count <= 1),
                }
            }
        }
        Ok(())
    }

    pub fn cleanup(&self) -> RepoResult<()> {
        self.cleanup_tags()?;
        self.cleanup_markers()?;
        self.cleanup_brief()?;
        self.cleanup_source()?;
        self.cleanup_collection()?;
        Ok(())
    }

    fn on_insert(&self, repo_id: RepoId, track: &Track) -> RepoResult<()> {
        self.insert_collection(repo_id, track)?;
        self.insert_source(repo_id, track)?;
        self.insert_brief(repo_id, track)?;
        self.insert_markers(repo_id, track)?;
        self.insert_tags(repo_id, track)?;
        Ok(())
    }

    fn on_delete(&self, repo_id: RepoId) -> RepoResult<()> {
        self.delete_tags(repo_id)?;
        self.delete_markers(repo_id)?;
        self.delete_brief(repo_id)?;
        self.delete_source(repo_id)?;
        self.delete_collection(repo_id)?;
        Ok(())
    }

    fn on_refresh(&self, repo_id: RepoId, track: &Track) -> RepoResult<()> {
        self.on_delete(repo_id)?;
        self.on_insert(repo_id, track)?;
        Ok(())
    }

    pub fn refresh_entity(&self, entity: &TrackEntity) -> RepoResult<RepoId> {
        let uid = &entity.hdr.uid;
        match self.resolve_repo_id(uid)? {
            Some(repo_id) => {
                self.on_refresh(repo_id, &entity.body)?;
                Ok(repo_id)
            }
            None => Err(failure::format_err!("Entity not found: {}", uid)),
        }
    }

    pub fn after_entity_inserted(&self, entity: &TrackEntity) -> RepoResult<RepoId> {
        let uid = &entity.hdr.uid;
        match self.resolve_repo_id(uid)? {
            Some(repo_id) => {
                self.on_insert(repo_id, &entity.body)?;
                Ok(repo_id)
            }
            None => Err(failure::format_err!("Entity not found: {}", uid)),
        }
    }

    pub fn before_entity_updated_or_removed(&self, uid: &EntityUid) -> RepoResult<RepoId> {
        match self.resolve_repo_id(uid)? {
            Some(repo_id) => {
                self.on_delete(repo_id)?;
                Ok(repo_id)
            }
            None => Err(failure::format_err!("Entity not found: {}", uid)),
        }
    }

    pub fn after_entity_updated(&self, repo_id: RepoId, track: &Track) -> RepoResult<()> {
        self.on_insert(repo_id, track)?;
        Ok(())
    }
}

impl<'a> EntityRepo for RepositoryHelper<'a> {
    fn resolve_repo_id(&self, uid: &EntityUid) -> RepoResult<Option<RepoId>> {
        tbl_track::table
            .select(tbl_track::id)
            .filter(tbl_track::uid.eq(uid.as_ref()))
            .first::<RepoId>(self.connection)
            .optional()
            .map_err(Into::into)
    }
}
