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

mod search;

use diesel::dsl::count_star;
use search::{TrackSearchBoxedExpressionBuilder as _, TrackSearchQueryTransform as _};

use crate::{
    db::{
        media_source::{schema::*, subselect as media_source_subselect},
        track::{models::*, schema::*, *},
    },
    prelude::*,
};

use aoide_core::{
    entity::{EntityHeader, EntityRevision, EntityUid},
    media::Source,
    tag::*,
    track::{actor::Actor, cue::Cue, title::Title, *},
    util::clock::*,
};

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::source::{RecordId as MediaSourceId, Repo as _},
    track::*,
};

fn load_track_and_album_titles(
    db: &crate::Connection<'_>,
    id: RecordId,
) -> RepoResult<(Vec<Title>, Vec<Title>)> {
    use crate::db::track_title::{models::*, schema::*, *};
    // TODO: Optimize
    let queryables = track_title::table
        .filter(track_title::track_id.eq(RowId::from(id)))
        .load::<QueryableRecord>(db.as_ref())
        .map_err(repo_error)?;
    let (mut track_titles, mut album_titles) = (
        Vec::with_capacity(queryables.len()),
        Vec::with_capacity(queryables.len()),
    );
    for queryable in queryables {
        let (_, record) = queryable.into();
        let Record {
            track_id: _,
            scope,
            title,
        } = record;
        match scope {
            Scope::Track => {
                track_titles.push(title);
            }
            Scope::Album => {
                album_titles.push(title);
            }
        }
    }
    Ok((track_titles, album_titles))
}

fn delete_track_and_album_titles(
    db: &crate::Connection<'_>,
    track_id: RecordId,
) -> RepoResult<usize> {
    use crate::db::track_title::schema::*;
    diesel::delete(track_title::table.filter(track_title::track_id.eq(RowId::from(track_id))))
        .execute(db.as_ref())
        .map_err(repo_error)
}

fn insert_track_and_album_titles(
    db: &crate::Connection<'_>,
    track_id: RecordId,
    track_titles: &[Title],
    album_titles: &[Title],
) -> RepoResult<()> {
    use crate::db::track_title::{models::*, schema::*};
    for track_title in track_titles {
        let insertable = InsertableRecord::bind(track_id, Scope::Track, track_title);
        diesel::insert_into(track_title::table)
            .values(&insertable)
            .execute(db.as_ref())
            .map_err(repo_error)?;
    }
    for album_title in album_titles {
        let insertable = InsertableRecord::bind(track_id, Scope::Album, album_title);
        diesel::insert_into(track_title::table)
            .values(&insertable)
            .execute(db.as_ref())
            .map_err(repo_error)?;
    }
    Ok(())
}

fn update_track_and_album_titles(
    db: &crate::Connection<'_>,
    track_id: RecordId,
    track_titles: &[Title],
    album_titles: &[Title],
) -> RepoResult<()> {
    // TODO: Is this preliminary check effective?
    let (old_track_titles, old_album_titles) = load_track_and_album_titles(db, track_id)?;
    if (&old_track_titles[..], &old_album_titles[..]) == (track_titles, album_titles) {
        log::debug!("Keeping unmodified track/album titles");
        return Ok(());
    }
    delete_track_and_album_titles(db, track_id)?;
    insert_track_and_album_titles(db, track_id, track_titles, album_titles)?;
    Ok(())
}

fn load_track_and_album_actors(
    db: &crate::Connection<'_>,
    id: RecordId,
) -> RepoResult<(Vec<Actor>, Vec<Actor>)> {
    use crate::db::track_actor::{models::*, schema::*, *};
    // TODO: Optimize
    let queryables = track_actor::table
        .filter(track_actor::track_id.eq(RowId::from(id)))
        .load::<QueryableRecord>(db.as_ref())
        .map_err(repo_error)?;
    let (mut track_actors, mut album_actors) = (
        Vec::with_capacity(queryables.len()),
        Vec::with_capacity(queryables.len()),
    );
    for queryable in queryables {
        let (_, record) = queryable.into();
        let Record {
            track_id: _,
            scope,
            actor,
        } = record;
        match scope {
            Scope::Track => {
                track_actors.push(actor);
            }
            Scope::Album => {
                album_actors.push(actor);
            }
        }
    }
    Ok((track_actors, album_actors))
}

fn delete_track_and_album_actors(
    db: &crate::Connection<'_>,
    track_id: RecordId,
) -> RepoResult<usize> {
    use crate::db::track_actor::schema::*;
    diesel::delete(track_actor::table.filter(track_actor::track_id.eq(RowId::from(track_id))))
        .execute(db.as_ref())
        .map_err(repo_error)
}

fn insert_track_and_album_actors(
    db: &crate::Connection<'_>,
    track_id: RecordId,
    track_actors: &[Actor],
    album_actors: &[Actor],
) -> RepoResult<()> {
    use crate::db::track_actor::{models::*, schema::*};
    for track_actor in track_actors {
        let insertable = InsertableRecord::bind(track_id, Scope::Track, track_actor);
        diesel::insert_into(track_actor::table)
            .values(&insertable)
            .execute(db.as_ref())
            .map_err(repo_error)?;
    }
    for album_actor in album_actors {
        let insertable = InsertableRecord::bind(track_id, Scope::Album, album_actor);
        diesel::insert_into(track_actor::table)
            .values(&insertable)
            .execute(db.as_ref())
            .map_err(repo_error)?;
    }
    Ok(())
}

fn update_track_and_album_actors(
    db: &crate::Connection<'_>,
    track_id: RecordId,
    track_actors: &[Actor],
    album_actors: &[Actor],
) -> RepoResult<()> {
    // TODO: Is this preliminary check effective?
    let (old_track_actors, old_album_actors) = load_track_and_album_actors(db, track_id)?;
    if (&old_track_actors[..], &old_album_actors[..]) == (track_actors, album_actors) {
        log::debug!("Keeping unmodified track/album actors");
        return Ok(());
    }
    delete_track_and_album_actors(db, track_id)?;
    insert_track_and_album_actors(db, track_id, track_actors, album_actors)?;
    Ok(())
}

fn load_track_cues(db: &crate::Connection<'_>, track_id: RecordId) -> RepoResult<Vec<Cue>> {
    use crate::db::track_cue::{models::*, schema::*, *};
    track_cue::table
        .filter(track_cue::track_id.eq(RowId::from(track_id)))
        .load::<QueryableRecord>(db.as_ref())
        .map_err(repo_error)
        .map(|queryables| {
            queryables
                .into_iter()
                .map(Into::into)
                .map(|(_, record)| {
                    let Record { track_id: _, cue } = record;
                    cue
                })
                .collect()
        })
}

fn delete_track_cues(db: &crate::Connection<'_>, track_id: RecordId) -> RepoResult<usize> {
    use crate::db::track_cue::schema::*;
    diesel::delete(track_cue::table.filter(track_cue::track_id.eq(RowId::from(track_id))))
        .execute(db.as_ref())
        .map_err(repo_error)
}

fn insert_track_cues(
    db: &crate::Connection<'_>,
    track_id: RecordId,
    cues: &[Cue],
) -> RepoResult<()> {
    use crate::db::track_cue::{models::*, schema::*};
    for cue in cues {
        let insertable = InsertableRecord::bind(track_id, cue);
        diesel::insert_into(track_cue::table)
            .values(&insertable)
            .execute(db.as_ref())
            .map_err(repo_error)?;
    }
    Ok(())
}

fn update_track_cues(
    db: &crate::Connection<'_>,
    track_id: RecordId,
    cues: &[Cue],
) -> RepoResult<()> {
    // TODO: Is this preliminary check effective?
    let old_cues = load_track_cues(db, track_id)?;
    if old_cues == cues {
        log::debug!("Keeping unmodified track cues");
        return Ok(());
    }
    delete_track_cues(db, track_id)?;
    insert_track_cues(db, track_id, cues)?;
    Ok(())
}

fn load_track_tags(db: &crate::Connection<'_>, track_id: RecordId) -> RepoResult<Tags> {
    use crate::db::track_tag::{models::*, schema::*};
    track_tag::table
        .filter(track_tag::track_id.eq(RowId::from(track_id)))
        .order_by(track_tag::facet)
        .load::<QueryableRecord>(db.as_ref())
        .map_err(repo_error)
        .map(|queryables| {
            // TODO: Optimize
            let mut tags = Tags::from(TagsMap::new());
            for queryable in queryables {
                let (_, record) = queryable.into();
                let (key, tag) = record.into();
                tags.insert(key, tag);
            }
            tags
        })
}

fn delete_track_tags(db: &crate::Connection<'_>, track_id: RecordId) -> RepoResult<usize> {
    use crate::db::track_tag::schema::*;
    diesel::delete(track_tag::table.filter(track_tag::track_id.eq(RowId::from(track_id))))
        .execute(db.as_ref())
        .map_err(repo_error)
}

fn insert_track_tags(
    db: &crate::Connection<'_>,
    track_id: RecordId,
    tags: &Tags,
) -> RepoResult<()> {
    use crate::db::track_tag::{models::*, schema::*};
    for (facet_key, plain_tags) in tags.as_ref() {
        for plain_tag in plain_tags {
            let insertable = InsertableRecord::bind(track_id, facet_key.as_ref(), plain_tag);
            diesel::insert_into(track_tag::table)
                .values(&insertable)
                .execute(db.as_ref())
                .map_err(repo_error)?;
        }
    }
    Ok(())
}

fn update_track_tags(
    db: &crate::Connection<'_>,
    track_id: RecordId,
    tags: &Tags,
) -> RepoResult<()> {
    // TODO: Is this preliminary check effective?
    let old_tags = load_track_tags(db, track_id)?;
    if &old_tags == tags {
        log::debug!("Keeping unmodified track tags");
        return Ok(());
    }
    delete_track_tags(db, track_id)?;
    insert_track_tags(db, track_id, tags)?;
    Ok(())
}

fn preload_entity(
    db: &crate::Connection<'_>,
    id: RecordId,
    media_source: Source,
) -> RepoResult<EntityPreload> {
    let (track_titles, album_titles) = load_track_and_album_titles(db, id)?;
    let (track_actors, album_actors) = load_track_and_album_actors(db, id)?;
    Ok(EntityPreload {
        media_source,
        album_actors,
        album_titles,
        cues: load_track_cues(db, id)?,
        tags: load_track_tags(db, id)?,
        track_actors,
        track_titles,
    })
}

impl<'db> EntityRepo for crate::Connection<'db> {
    fn resolve_track_entity_revision(
        &self,
        uid: &EntityUid,
    ) -> RepoResult<(RecordHeader, EntityRevision)> {
        track::table
            .select((
                track::row_id,
                track::row_created_ms,
                track::row_updated_ms,
                track::entity_rev,
            ))
            .filter(track::entity_uid.eq(uid.as_ref()))
            .first::<(RowId, TimestampMillis, TimestampMillis, i64)>(self.as_ref())
            .map_err(repo_error)
            .map(|(row_id, row_created_ms, row_updated_ms, entity_rev)| {
                let header = RecordHeader {
                    id: row_id.into(),
                    created_at: DateTime::new_timestamp_millis(row_created_ms),
                    updated_at: DateTime::new_timestamp_millis(row_updated_ms),
                };
                (header, entity_revision_from_sql(entity_rev))
            })
    }

    fn insert_track_entity(
        &self,
        created_at: DateTime,
        media_source_id: MediaSourceId,
        created_entity: &Entity,
    ) -> RepoResult<RecordId> {
        let record = InsertableRecord::bind(created_at, media_source_id, created_entity);
        let query = diesel::insert_into(track::table).values(&record);
        let _rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert_eq!(1, _rows_affected);
        let id = self.resolve_track_id(&created_entity.hdr.uid)?;
        insert_track_and_album_titles(
            self,
            id,
            &created_entity.body.titles,
            &created_entity.body.album.titles,
        )?;
        insert_track_and_album_actors(
            self,
            id,
            &created_entity.body.actors,
            &created_entity.body.album.actors,
        )?;
        insert_track_cues(self, id, &created_entity.body.cues)?;
        insert_track_tags(self, id, &created_entity.body.tags)?;
        Ok(id)
    }

    fn update_track_entity(
        &self,
        id: RecordId,
        updated_at: DateTime,
        media_source_id: MediaSourceId,
        updated_entity: &Entity,
    ) -> RepoResult<()> {
        let record = UpdatableRecord::bind(
            updated_at,
            updated_entity.hdr.rev,
            media_source_id,
            &updated_entity.body,
        );
        let target = track::table.filter(track::row_id.eq(RowId::from(id)));
        let query = diesel::update(target).set(&record);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        update_track_and_album_titles(
            self,
            id,
            &updated_entity.body.titles,
            &updated_entity.body.album.titles,
        )?;
        update_track_and_album_actors(
            self,
            id,
            &updated_entity.body.actors,
            &updated_entity.body.album.actors,
        )?;
        update_track_cues(self, id, &updated_entity.body.cues)?;
        update_track_tags(self, id, &updated_entity.body.tags)?;
        Ok(())
    }

    fn delete_track_entity(&self, id: RecordId) -> RepoResult<()> {
        delete_track_and_album_titles(self, id)?;
        delete_track_and_album_actors(self, id)?;
        delete_track_cues(self, id)?;
        delete_track_tags(self, id)?;
        let target = track::table.filter(track::row_id.eq(RowId::from(id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn load_track_entity(&self, id: RecordId) -> RepoResult<(RecordHeader, Entity)> {
        let queryable = track::table
            .filter(track::row_id.eq(RowId::from(id)))
            .first::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)?;
        let (_, media_source) = self.load_media_source(queryable.media_source_id.into())?;
        let preload = preload_entity(self, id, media_source)?;
        Ok(load_repo_entity(preload, queryable))
    }

    fn load_track_entity_by_uid(&self, uid: &EntityUid) -> RepoResult<(RecordHeader, Entity)> {
        let queryable = track::table
            .filter(track::entity_uid.eq(uid.as_ref()))
            .first::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)?;
        let (_, media_source) = self.load_media_source(queryable.media_source_id.into())?;
        let preload = preload_entity(self, queryable.id.into(), media_source)?;
        Ok(load_repo_entity(preload, queryable))
    }

    fn load_track_entity_by_media_source_uri(
        &self,
        collection_id: CollectionId,
        uri: &str,
    ) -> RepoResult<(MediaSourceId, RecordHeader, Entity)> {
        let media_source_id_subselect = media_source_subselect::filter_by_uri_predicate(
            collection_id,
            StringPredicateBorrowed::Equals(uri),
        );
        let queryable = track::table
            .filter(track::media_source_id.eq_any(media_source_id_subselect))
            .first::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)?;
        let media_source_id = queryable.media_source_id.into();
        let (_, media_source) = self.load_media_source(media_source_id)?;
        let preload = preload_entity(self, queryable.id.into(), media_source)?;
        let (record_header, entity) = load_repo_entity(preload, queryable);
        Ok((media_source_id, record_header, entity))
    }

    fn resolve_track_entity_header_by_media_source_uri(
        &self,
        collection_id: CollectionId,
        uri: &str,
    ) -> RepoResult<(MediaSourceId, RecordHeader, EntityHeader)> {
        let media_source_id_subselect = media_source_subselect::filter_by_uri_predicate(
            collection_id,
            StringPredicateBorrowed::Equals(uri),
        );
        let queryable = track::table
            .filter(track::media_source_id.eq_any(media_source_id_subselect))
            .first::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)?;
        Ok(queryable.into())
    }

    fn list_track_entities(
        &self,
        pagination: &Pagination,
    ) -> RepoResult<Vec<(RecordHeader, Entity)>> {
        let mut target = track::table
            .order_by(track::row_updated_ms.desc())
            .into_boxed();

        // Pagination
        target = apply_pagination(target, pagination);

        let queryables = target
            .load::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)?;
        let mut loaded_repo_entities = Vec::with_capacity(queryables.len());
        for queryable in queryables {
            let media_source_id = queryable.media_source_id.into();
            let (_, media_source) = self.load_media_source(media_source_id)?;
            let preload = preload_entity(self, queryable.id.into(), media_source)?;
            loaded_repo_entities.push(load_repo_entity(preload, queryable));
        }
        Ok(loaded_repo_entities)
    }

    fn replace_collected_track_by_media_source_uri(
        &self,
        collection_id: CollectionId,
        replace_mode: ReplaceMode,
        track: Track,
    ) -> RepoResult<ReplaceOutcome> {
        let loaded = self
            .load_track_entity_by_media_source_uri(collection_id, &track.media_source.uri)
            .optional()?;
        if let Some((media_source_id, record_header, mut entity)) = loaded {
            // Update existing entry
            let id = record_header.id;
            if replace_mode == ReplaceMode::CreateOnly {
                return Ok(ReplaceOutcome::NotUpdated(id, track));
            }
            let updated_at = DateTime::now_utc();
            if entity.body == track {
                return Ok(ReplaceOutcome::Unchanged(id, entity));
            }
            if track.media_source != entity.body.media_source {
                self.update_media_source(media_source_id, updated_at, &track.media_source)?;
            }
            let current_rev = entity.hdr.rev;
            entity.hdr.rev = current_rev.next();
            entity.body = track;
            self.update_track_entity(id, updated_at, media_source_id, &entity)?;
            Ok(ReplaceOutcome::Updated(id, entity))
        } else {
            // Create new entry
            if replace_mode == ReplaceMode::UpdateOnly {
                return Ok(ReplaceOutcome::NotCreated(track));
            }
            let created_at = DateTime::now_utc();
            let media_source_id = self
                .insert_media_source(created_at, collection_id, &track.media_source)?
                .id;
            let entity = Entity::new(EntityHeader::initial_random(), track);
            let id = self.insert_track_entity(created_at, media_source_id, &entity)?;
            Ok(ReplaceOutcome::Created(id, entity))
        }
    }

    fn purge_tracks_by_media_source_uri_predicate(
        &self,
        collection_id: CollectionId,
        uri_predicate: StringPredicateBorrowed<'_>,
    ) -> RepoResult<usize> {
        let media_source_id_subselect =
            media_source_subselect::filter_by_uri_predicate(collection_id, uri_predicate);
        let row_ids = track::table
            .select(track::row_id)
            .filter(track::media_source_id.eq_any(media_source_id_subselect))
            .load::<RowId>(self.as_ref())
            .map_err(repo_error)?;
        let total_count = row_ids.len();
        for row_id in row_ids {
            self.delete_track_entity(row_id.into())?;
        }
        Ok(total_count)
    }

    fn search_collected_tracks(
        &self,
        collection_id: CollectionId,
        pagination: &Pagination,
        filter: Option<SearchFilter>,
        ordering: Vec<SortOrder>,
        collector: &mut dyn ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
    ) -> RepoResult<usize> {
        let mut query =
            track::table
                .inner_join(media_source::table)
                .select(track::all_columns)
                .filter(track::media_source_id.eq_any(
                    media_source_subselect::filter_by_collection_id(collection_id),
                ))
                .into_boxed();

        if let Some(ref filter) = filter {
            query = query.filter(filter.build_expression());
        }

        for sort_order in &ordering {
            query = sort_order.apply_to_query(query);
        }
        // Finally order by PK to preserve the relative order of results
        // even if no sorting was requested.
        query = query.then_order_by(track::row_id);

        // Pagination
        query = apply_pagination(query, pagination);

        let queryables = query
            .load::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)?;
        let count = queryables.len();
        collector.reserve(count);
        for queryable in queryables {
            let media_source_id = queryable.media_source_id.into();
            let (_, media_source) = self.load_media_source(media_source_id)?;
            let preload = preload_entity(self, queryable.id.into(), media_source)?;
            let (record_header, entity) = load_repo_entity(preload, queryable);
            collector.collect(record_header, entity);
        }
        Ok(count)
    }

    fn count_collected_tracks(&self, collection_id: CollectionId) -> RepoResult<u64> {
        track::table
            .select(count_star())
            .filter(
                track::media_source_id.eq_any(media_source_subselect::filter_by_collection_id(
                    collection_id,
                )),
            )
            .first::<i64>(self.as_ref())
            .map_err(repo_error)
            .map(|count| {
                debug_assert!(count >= 0);
                count as u64
            })
    }
}
