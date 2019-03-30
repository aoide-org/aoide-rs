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

use crate::{
    api::{
        collection::Collections,
        entity::{EntityStorage, EntityStorageResult, StorageId},
    },
    storage::collection::{schema::tbl_collection, CollectionRepository},
};

use diesel;

use failure::Error;

///////////////////////////////////////////////////////////////////////
/// TrackRepositoryHelper
///////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct TrackRepositoryHelper<'a> {
    connection: &'a diesel::SqliteConnection,
}

impl<'a> TrackRepositoryHelper<'a> {
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
                collection_repo.insert_entity(&collection_entity)?;
                recreated_collections.push(collection_entity);
            }
        }
        Ok(recreated_collections)
    }

    fn cleanup_source(&self) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_source::table
                .filter(aux_track_source::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_source(&self, track_id: StorageId) -> Result<(), Error> {
        let query =
            diesel::delete(aux_track_source::table.filter(aux_track_source::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_source(&self, track_id: StorageId, track: &Track) -> Result<(), Error> {
        for source in &track.sources {
            let insertable = InsertableTracksSource::bind(track_id, source);
            let query = diesel::insert_into(aux_track_source::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_collection(&self) -> Result<(), Error> {
        let query =
            diesel::delete(aux_track_collection::table.filter(
                aux_track_collection::track_id.ne_all(tbl_track::table.select(tbl_track::id)),
            ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_collection(&self, track_id: StorageId) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_collection::table.filter(aux_track_collection::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_collection(&self, track_id: StorageId, track: &Track) -> Result<(), Error> {
        for collection in &track.collections {
            let insertable = InsertableTracksCollection::bind(track_id, collection);
            let query = diesel::insert_into(aux_track_collection::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_brief(&self) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_brief::table
                .filter(aux_track_brief::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_brief(&self, track_id: StorageId) -> Result<(), Error> {
        let query =
            diesel::delete(aux_track_brief::table.filter(aux_track_brief::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_brief(&self, track_id: StorageId, track: &Track) -> Result<(), Error> {
        let insertable = InsertableTracksBrief::bind(track_id, track);
        let query = diesel::insert_into(aux_track_brief::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn cleanup_tags(&self) -> Result<(), Error> {
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

    fn delete_tags(&self, track_id: StorageId) -> Result<(), Error> {
        diesel::delete(aux_track_tag::table.filter(aux_track_tag::track_id.eq(track_id)))
            .execute(self.connection)?;
        Ok(())
    }

    fn resolve_tag_label(&self, label: &Label) -> Result<StorageId, Error> {
        debug_assert!(label.is_valid());
        loop {
            match aux_tag_label::table
                .select(aux_tag_label::id)
                .filter(aux_tag_label::label.eq(label.as_ref()))
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

    fn resolve_tag_facet(&self, facet: &Facet) -> Result<StorageId, Error> {
        debug_assert!(facet.is_valid());
        loop {
            match aux_tag_facet::table
                .select(aux_tag_facet::id)
                .filter(aux_tag_facet::facet.eq(facet.as_ref()))
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

    fn insert_tags(&self, track_id: StorageId, track: &Track) -> Result<(), Error> {
        for tag in &track.tags.plain {
            if let Some(label) = tag.label() {
                let label_id = self.resolve_tag_label(label)?;
                let insertable =
                    InsertableTracksTag::bind(track_id, None, Some(label_id), tag.score());
                match diesel::insert_into(aux_track_tag::table)
                    .values(&insertable)
                    .execute(self.connection)
                {
                    Err(err) => log::error!(
                        "Failed to insert plain tag {:?} for track {}: {}",
                        tag,
                        track_id,
                        err
                    ),
                    Ok(count) => debug_assert!(count == 1),
                }
            }
        }
        for tag in &track.tags.faceted {
            let facet_id = self.resolve_tag_facet(tag.facet())?;
            let label_id = if let Some(label) = tag.label() {
                Some(self.resolve_tag_label(label)?)
            } else {
                None
            };
            let insertable =
                InsertableTracksTag::bind(track_id, Some(facet_id), label_id, tag.score());
            match diesel::insert_into(aux_track_tag::table)
                .values(&insertable)
                .execute(self.connection)
            {
                Err(err) => log::error!(
                    "Failed to insert faceted tag {:?} for track {}: {}",
                    tag,
                    track_id,
                    err
                ),
                Ok(count) => debug_assert!(count == 1),
            }
        }
        Ok(())
    }

    pub fn cleanup(&self) -> Result<(), Error> {
        self.cleanup_tags()?;
        self.cleanup_brief()?;
        self.cleanup_source()?;
        self.cleanup_collection()?;
        Ok(())
    }

    fn on_insert(&self, storage_id: StorageId, track: &Track) -> Result<(), Error> {
        self.insert_collection(storage_id, track)?;
        self.insert_source(storage_id, track)?;
        self.insert_brief(storage_id, track)?;
        self.insert_tags(storage_id, track)?;
        Ok(())
    }

    fn on_delete(&self, storage_id: StorageId) -> Result<(), Error> {
        self.delete_tags(storage_id)?;
        self.delete_brief(storage_id)?;
        self.delete_source(storage_id)?;
        self.delete_collection(storage_id)?;
        Ok(())
    }

    fn on_refresh(&self, storage_id: StorageId, track: &Track) -> Result<(), Error> {
        self.on_delete(storage_id)?;
        self.on_insert(storage_id, track)?;
        Ok(())
    }

    pub fn refresh_entity(&self, entity: &TrackEntity) -> Result<StorageId, Error> {
        let uid = entity.header().uid();
        match self.find_storage_id(&uid)? {
            Some(storage_id) => {
                self.on_refresh(storage_id, entity.body())?;
                Ok(storage_id)
            }
            None => Err(failure::format_err!("Entity not found: {}", uid)),
        }
    }

    pub fn after_entity_inserted(&self, entity: &TrackEntity) -> Result<StorageId, Error> {
        let uid = entity.header().uid();
        match self.find_storage_id(&uid)? {
            Some(storage_id) => {
                self.on_insert(storage_id, entity.body())?;
                Ok(storage_id)
            }
            None => Err(failure::format_err!("Entity not found: {}", uid)),
        }
    }

    pub fn before_entity_updated_or_removed(&self, uid: &EntityUid) -> Result<StorageId, Error> {
        match self.find_storage_id(uid)? {
            Some(storage_id) => {
                self.on_delete(storage_id)?;
                Ok(storage_id)
            }
            None => Err(failure::format_err!("Entity not found: {}", uid)),
        }
    }

    pub fn after_entity_updated(&self, storage_id: StorageId, track: &Track) -> Result<(), Error> {
        self.on_insert(storage_id, track)?;
        Ok(())
    }
}

impl<'a> EntityStorage for TrackRepositoryHelper<'a> {
    fn find_storage_id(&self, uid: &EntityUid) -> EntityStorageResult<Option<StorageId>> {
        tbl_track::table
            .select(tbl_track::id)
            .filter(tbl_track::uid.eq(uid.as_ref()))
            .first::<StorageId>(self.connection)
            .optional()
            .map_err(Into::into)
    }
}
