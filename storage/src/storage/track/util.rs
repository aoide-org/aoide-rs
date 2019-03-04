// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    core::collection::{Collection, CollectionEntity},
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

    fn cleanup_overview(&self) -> Result<(), Error> {
        let query =
            diesel::delete(aux_track_overview::table.filter(
                aux_track_overview::track_id.ne_all(tbl_track::table.select(tbl_track::id)),
            ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_overview(&self, track_id: StorageId) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_overview::table.filter(aux_track_overview::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_overview(&self, track_id: StorageId, track: &Track) -> Result<(), Error> {
        let insertable = InsertableTracksOverview::bind(track_id, track);
        let query = diesel::insert_into(aux_track_overview::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn cleanup_summary(&self) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_summary::table
                .filter(aux_track_summary::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_summary(&self, track_id: StorageId) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_summary::table.filter(aux_track_summary::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_summary(&self, track_id: StorageId, track: &Track) -> Result<(), Error> {
        let insertable = InsertableTracksSummary::bind(track_id, track);
        let query = diesel::insert_into(aux_track_summary::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
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

    fn cleanup_profile(&self) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_profile::table
                .filter(aux_track_profile::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_profile(&self, track_id: StorageId) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_profile::table.filter(aux_track_profile::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_profile(&self, track_id: StorageId, track: &Track) -> Result<(), Error> {
        if track.profile.is_some() {
            let insertable = InsertableTracksMusic::bind(track_id, track.profile.as_ref().unwrap());
            let query = diesel::insert_into(aux_track_profile::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_xref(&self) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_xref::table
                .filter(aux_track_xref::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_xref(&self, track_id: StorageId) -> Result<(), Error> {
        let query =
            diesel::delete(aux_track_xref::table.filter(aux_track_xref::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_xref(&self, track_id: StorageId, track: &Track) -> Result<(), Error> {
        for track_xref in &track.external_references {
            let insertable = InsertableTracksRef::bind(track_id, RefOrigin::Track, &track_xref);
            let query = diesel::replace_into(aux_track_xref::table).values(&insertable);
            query.execute(self.connection)?;
        }
        for actor in &track.actors {
            for actor_xref in &actor.external_references {
                let insertable =
                    InsertableTracksRef::bind(track_id, RefOrigin::TrackActor, &actor_xref);
                let query = diesel::replace_into(aux_track_xref::table).values(&insertable);
                query.execute(self.connection)?;
            }
        }
        if let Some(album) = track.album.as_ref() {
            for album_xref in &album.external_references {
                let insertable = InsertableTracksRef::bind(track_id, RefOrigin::Album, &album_xref);
                let query = diesel::replace_into(aux_track_xref::table).values(&insertable);
                query.execute(self.connection)?;
            }
            for actor in &album.actors {
                for actor_xref in &actor.external_references {
                    let insertable =
                        InsertableTracksRef::bind(track_id, RefOrigin::AlbumActor, &actor_xref);
                    let query = diesel::replace_into(aux_track_xref::table).values(&insertable);
                    query.execute(self.connection)?;
                }
            }
        }
        if let Some(release) = track.release.as_ref() {
            for release_xref in &release.external_references {
                let insertable =
                    InsertableTracksRef::bind(track_id, RefOrigin::Release, &release_xref);
                let query = diesel::replace_into(aux_track_xref::table).values(&insertable);
                query.execute(self.connection)?;
            }
        }
        Ok(())
    }

    fn cleanup_tag(&self) -> Result<(), Error> {
        // Orphaned tags
        diesel::delete(
            aux_track_tag::table
                .filter(aux_track_tag::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        )
        .execute(self.connection)?;
        // Orphaned tag terms
        diesel::delete(aux_track_tag_term::table.filter(
            aux_track_tag_term::id.ne_all(aux_track_tag::table.select(aux_track_tag::term_id)),
        ))
        .execute(self.connection)?;
        // Orphaned tag facets
        diesel::delete(aux_track_tag_facet::table.filter(
            aux_track_tag_facet::id.ne_all(aux_track_tag::table.select(aux_track_tag::facet_id)),
        ))
        .execute(self.connection)?;
        Ok(())
    }

    fn delete_tag(&self, track_id: StorageId) -> Result<(), Error> {
        diesel::delete(aux_track_tag::table.filter(aux_track_tag::track_id.eq(track_id)))
            .execute(self.connection)?;
        Ok(())
    }

    fn get_or_add_tag_term(&self, term: &str) -> Result<StorageId, Error> {
        debug_assert!(!term.is_empty());
        loop {
            match aux_track_tag_term::table
                .select(aux_track_tag_term::id)
                .filter(aux_track_tag_term::term.eq(term))
                .first(self.connection)
                .optional()?
            {
                Some(id) => return Ok(id),
                None => {
                    let insertable = InsertableTracksTagTerm::bind(term);
                    diesel::insert_or_ignore_into(aux_track_tag_term::table)
                        .values(&insertable)
                        .execute(self.connection)?;
                    // and retry...
                }
            }
        }
    }

    fn get_or_add_tag_facet(&self, facet: &str) -> Result<StorageId, Error> {
        debug_assert!(!facet.is_empty());
        debug_assert!(facet == facet.to_lowercase());
        loop {
            // TODO: End the expression with ".optional()?"" after removing Nullable from aux_track_tag_facet::id in schema
            // See also: https://github.com/diesel-rs/diesel/pull/1644
            match aux_track_tag_facet::table
                .select(aux_track_tag_facet::id)
                .filter(aux_track_tag_facet::facet.eq(facet))
                .first(self.connection)
            {
                Ok(Some(id)) => return Ok(id),
                Ok(None) | Err(diesel::NotFound) => {
                    let insertable = InsertableTracksTagFacet::bind(facet);
                    diesel::insert_or_ignore_into(aux_track_tag_facet::table)
                        .values(&insertable)
                        .execute(self.connection)?;
                    // and retry...
                }
                Err(e) => Err(e)?,
            }
        }
    }

    fn insert_tag(&self, track_id: StorageId, track: &Track) -> Result<(), Error> {
        for tag in &track.tags {
            let term_id = self.get_or_add_tag_term(tag.term())?;
            let facet_id = match tag.facet() {
                Some(facet) => Some(self.get_or_add_tag_facet(facet)?),
                None => None,
            };
            let insertable = InsertableTracksTag::bind(track_id, term_id, facet_id, tag.score());
            diesel::insert_into(aux_track_tag::table)
                .values(&insertable)
                .execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_comment(&self) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_comment::table
                .filter(aux_track_comment::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_comment(&self, track_id: StorageId) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_comment::table.filter(aux_track_comment::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_comment(&self, track_id: StorageId, track: &Track) -> Result<(), Error> {
        for comment in &track.comments {
            let insertable = InsertableTracksComment::bind(track_id, comment);
            let query = diesel::insert_into(aux_track_comment::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_rating(&self) -> Result<(), Error> {
        let query = diesel::delete(
            aux_track_rating::table
                .filter(aux_track_rating::track_id.ne_all(tbl_track::table.select(tbl_track::id))),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_rating(&self, track_id: StorageId) -> Result<(), Error> {
        let query =
            diesel::delete(aux_track_rating::table.filter(aux_track_rating::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_rating(&self, track_id: StorageId, track: &Track) -> Result<(), Error> {
        for rating in &track.ratings {
            let insertable = InsertableTracksRating::bind(track_id, rating);
            let query = diesel::insert_into(aux_track_rating::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    pub fn cleanup(&self) -> Result<(), Error> {
        self.cleanup_overview()?;
        self.cleanup_summary()?;
        self.cleanup_source()?;
        self.cleanup_collection()?;
        self.cleanup_profile()?;
        self.cleanup_xref()?;
        self.cleanup_tag()?;
        self.cleanup_comment()?;
        self.cleanup_rating()?;
        Ok(())
    }

    fn on_insert(&self, storage_id: StorageId, track: &Track) -> Result<(), Error> {
        self.insert_overview(storage_id, track)?;
        self.insert_summary(storage_id, track)?;
        self.insert_source(storage_id, track)?;
        self.insert_collection(storage_id, track)?;
        self.insert_profile(storage_id, track)?;
        self.insert_xref(storage_id, track)?;
        self.insert_tag(storage_id, track)?;
        self.insert_comment(storage_id, track)?;
        self.insert_rating(storage_id, track)?;
        Ok(())
    }

    fn on_delete(&self, storage_id: StorageId) -> Result<(), Error> {
        self.delete_overview(storage_id)?;
        self.delete_summary(storage_id)?;
        self.delete_source(storage_id)?;
        self.delete_collection(storage_id)?;
        self.delete_profile(storage_id)?;
        self.delete_xref(storage_id)?;
        self.delete_tag(storage_id)?;
        self.delete_comment(storage_id)?;
        self.delete_rating(storage_id)?;
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
