// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{collections::HashSet, time::Instant};

use anyhow::anyhow;
use diesel::prelude::*;
use nonicle::{Canonical, CanonicalizeInto as _};

use aoide_core::{
    EncodedEntityUid, Track, TrackBody, TrackEntity, TrackHeader, TrackUid,
    media::{
        Source,
        content::{ContentLink, ContentPath, ContentRevision},
    },
    tag::*,
    track::{
        actor::{Actor, ActorNamesSummarySplitter, Kind as ActorKind},
        cue::Cue,
        title::Title,
    },
    util::clock::*,
};
use aoide_core_api::{
    Pagination,
    filtering::StringPredicate,
    track::search::{Filter, Scope, SortOrder},
};
use aoide_repo::{
    CollectionId, MediaSourceId, OptionalRepoResult as _, RepoError, RepoResult,
    ReservableRecordCollector, TrackId,
    media::source::{CollectionRepo as _, Repo as _},
    track::{
        ActorRepo, CollectionRepo, EntityRepo, RecordHeader, RecordTrail, ReplaceMode,
        ReplaceOutcome, ReplaceParams,
    },
};

use crate::{
    Connection, RowId,
    db::{
        collection::schema::*,
        media_source::{
            schema::*,
            select_row_id_filtered_by_collection_id as select_media_source_id_filtered_by_collection_id,
            select_row_id_filtered_by_content_path_predicate as select_media_source_id_filtered_by_content_path_predicate,
        },
        track::{models::*, schema::*, *},
        view_track_search::{
            models::{QueryableRecord as SearchQueryableRecord, load_repo_entity},
            schema::*,
        },
    },
    repo_error,
    util::{
        entity::{decode_entity_header, decode_entity_revision},
        pagination_to_limit_offset,
    },
};

mod search;
use self::search::{TrackSearchExpressionBoxedBuilder as _, TrackSearchQueryTransform as _};

// TODO: Define a dedicated return type
#[expect(clippy::type_complexity)]
fn load_track_and_album_titles(
    db: &mut Connection<'_>,
    id: TrackId,
) -> RepoResult<(Canonical<Vec<Title>>, Canonical<Vec<Title>>)> {
    use crate::db::track_title::{models::*, schema::*, *};

    let query = track_title::table
        .filter(track_title::track_id.eq(RowId::from(id)))
        // Establish canonical ordering on load!
        .order_by((track_title::scope, track_title::kind, track_title::name));
    let rows = query
        .load_iter::<QueryableRecord, _>(db.as_mut())
        .map_err(repo_error)?;

    let mut track_titles = Vec::with_capacity(rows.size_hint().0);
    let mut album_titles = Vec::with_capacity(rows.size_hint().0);
    for row in rows {
        let (_, record) = row
            .map_err(repo_error)?
            .try_into()
            .map_err(RepoError::Other)?;
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

    // The ordering returned by the SQL database might differ from the
    // canonical ordering due to custom text column collations!
    let track_titles = track_titles.canonicalize_into();
    let album_titles = album_titles.canonicalize_into();
    Ok((track_titles, album_titles))
}

fn delete_track_and_album_titles(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
) -> RepoResult<usize> {
    use crate::db::track_title::schema::*;
    diesel::delete(track_title::table.filter(track_title::track_id.eq(RowId::from(track_id))))
        .execute(db.as_mut())
        .map_err(repo_error)
}

fn insert_track_and_album_titles(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    track_titles: Canonical<&[Title]>,
    album_titles: Canonical<&[Title]>,
) -> RepoResult<()> {
    use crate::db::track_title::{models::*, schema::*};
    for track_title in *track_titles.as_ref() {
        let insertable = InsertableRecord::bind(track_id, Scope::Track, track_title);
        insertable
            .insert_into(track_title::table)
            .execute(db.as_mut())
            .map_err(repo_error)?;
    }
    for album_title in *album_titles.as_ref() {
        let insertable = InsertableRecord::bind(track_id, Scope::Album, album_title);
        insertable
            .insert_into(track_title::table)
            .execute(db.as_mut())
            .map_err(repo_error)?;
    }
    Ok(())
}

fn update_track_and_album_titles(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    new_track_titles: Canonical<&[Title]>,
    new_album_titles: Canonical<&[Title]>,
) -> RepoResult<()> {
    let (old_track_titles, old_album_titles) = load_track_and_album_titles(db, track_id)?;
    if (
        old_track_titles.as_canonical_slice(),
        old_album_titles.as_canonical_slice(),
    ) == (new_track_titles, new_album_titles)
    {
        log::debug!("Keeping unmodified track/album titles");
        return Ok(());
    }
    delete_track_and_album_titles(db, track_id)?;
    insert_track_and_album_titles(db, track_id, new_track_titles, new_album_titles)?;
    Ok(())
}

// TODO: Define a dedicated return type
#[expect(clippy::type_complexity)]
fn load_track_and_album_actors(
    db: &mut crate::Connection<'_>,
    id: TrackId,
) -> RepoResult<(Canonical<Vec<Actor>>, Canonical<Vec<Actor>>)> {
    use crate::db::track_actor::{models::*, schema::*, *};

    let query = track_actor::table
        .filter(track_actor::track_id.eq(RowId::from(id)))
        // Establish canonical ordering on load!
        .order_by((
            track_actor::scope,
            track_actor::role,
            track_actor::kind,
            track_actor::name,
        ));

    let rows = query
        .load_iter::<QueryableRecord, _>(db.as_mut())
        .map_err(repo_error)?;

    let mut track_actors = Vec::with_capacity(rows.size_hint().0);
    let mut album_actors = Vec::with_capacity(rows.size_hint().0);
    for row in rows {
        let (_, record) = row
            .map_err(repo_error)?
            .try_into()
            .map_err(RepoError::Other)?;
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

    // The ordering returned by the SQL database might differ from the
    // canonical ordering due to custom text column collations!
    let track_actors = track_actors.canonicalize_into();
    let album_actors = album_actors.canonicalize_into();
    Ok((track_actors, album_actors))
}

fn delete_track_and_album_actors(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
) -> RepoResult<usize> {
    use crate::db::track_actor::schema::*;
    diesel::delete(track_actor::table.filter(track_actor::track_id.eq(RowId::from(track_id))))
        .execute(db.as_mut())
        .map_err(repo_error)
}

fn insert_track_and_album_actors(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    track_actors: Canonical<&[Actor]>,
    album_actors: Canonical<&[Actor]>,
) -> RepoResult<()> {
    use crate::db::track_actor::{models::*, schema::*};
    for track_actor in *track_actors.as_ref() {
        let insertable = InsertableRecord::bind(track_id, Scope::Track, track_actor);
        insertable
            .insert_into(track_actor::table)
            .execute(db.as_mut())
            .map_err(repo_error)?;
    }
    for album_actor in *album_actors.as_ref() {
        let insertable = InsertableRecord::bind(track_id, Scope::Album, album_actor);
        insertable
            .insert_into(track_actor::table)
            .execute(db.as_mut())
            .map_err(repo_error)?;
    }
    Ok(())
}

fn update_track_and_album_actors(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    new_track_actors: Canonical<&[Actor]>,
    new_album_actors: Canonical<&[Actor]>,
) -> RepoResult<()> {
    let (old_track_actors, old_album_actors) = load_track_and_album_actors(db, track_id)?;
    if (
        old_track_actors.as_canonical_slice(),
        old_album_actors.as_canonical_slice(),
    ) == (new_track_actors, new_album_actors)
    {
        log::debug!("Keeping unmodified track/album actors");
        return Ok(());
    }
    delete_track_and_album_actors(db, track_id)?;
    insert_track_and_album_actors(db, track_id, new_track_actors, new_album_actors)?;
    Ok(())
}

fn load_track_cues(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
) -> RepoResult<Canonical<Vec<Cue>>> {
    use crate::db::track_cue::{models::*, schema::*, *};
    let query = track_cue::table
        .filter(track_cue::track_id.eq(RowId::from(track_id)))
        // Establish canonical ordering on load!
        .order_by((track_cue::bank_idx, track_cue::slot_idx));
    let rows = query
        .load_iter::<QueryableRecord, _>(db.as_mut())
        .map_err(repo_error)?;
    let cues = rows
        .map(|row| {
            row.map_err(repo_error).map(|queryable| {
                let (_, Record { track_id: _, cue }) = queryable.into();
                cue
            })
        })
        .collect::<RepoResult<_>>()?;
    Ok(Canonical::tie(cues))
}

fn delete_track_cues(db: &mut crate::Connection<'_>, track_id: TrackId) -> RepoResult<usize> {
    use crate::db::track_cue::schema::*;
    diesel::delete(track_cue::table.filter(track_cue::track_id.eq(RowId::from(track_id))))
        .execute(db.as_mut())
        .map_err(repo_error)
}

fn insert_track_cues(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    cues: Canonical<&[Cue]>,
) -> RepoResult<()> {
    use crate::db::track_cue::{models::*, schema::*};
    for cue in *cues.as_ref() {
        let insertable = InsertableRecord::bind(track_id, cue);
        insertable
            .insert_into(track_cue::table)
            .execute(db.as_mut())
            .map_err(repo_error)?;
    }
    Ok(())
}

fn update_track_cues(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    new_cues: Canonical<&[Cue]>,
) -> RepoResult<()> {
    let old_cues = load_track_cues(db, track_id)?;
    if old_cues.as_canonical_slice() == new_cues {
        log::debug!("Keeping unmodified track cues");
        return Ok(());
    }
    delete_track_cues(db, track_id)?;
    insert_track_cues(db, track_id, new_cues)?;
    Ok(())
}

fn load_track_tags(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
) -> RepoResult<Canonical<Tags<'static>>> {
    use crate::db::track_tag::{models::*, schema::*};

    let query = track_tag::table
        .filter(track_tag::track_id.eq(RowId::from(track_id)))
        // Establish canonical ordering on load!
        .order_by((track_tag::facet, track_tag::label, track_tag::score.desc()));
    let rows = query
        .load_iter::<QueryableRecord, _>(db.as_mut())
        .map_err(repo_error)?;

    let mut plain_tags = vec![];
    let mut facets: Vec<FacetedTags<'_>> = vec![];
    for row in rows {
        let (_, record) = row.map_err(repo_error)?.into();
        let (facet_id, tag) = record.into();
        if let Some(facet_id) = facet_id {
            if let Some(faceted_tags) = facets.last_mut() {
                if faceted_tags.facet_id == facet_id {
                    faceted_tags.tags.push(tag);
                    continue;
                }
            }
            facets.push(FacetedTags {
                facet_id,
                tags: vec![tag],
            });
        } else {
            plain_tags.push(tag);
        }
    }

    let tags = Tags {
        plain: plain_tags,
        facets,
    };
    // The ordering returned by the SQL database might differ from the
    // canonical ordering due to custom text column collations!
    Ok(tags.canonicalize_into())
}

fn delete_track_tags(db: &mut crate::Connection<'_>, track_id: TrackId) -> RepoResult<usize> {
    use crate::db::track_tag::schema::*;
    diesel::delete(track_tag::table.filter(track_tag::track_id.eq(RowId::from(track_id))))
        .execute(db.as_mut())
        .map_err(repo_error)
}

fn insert_track_tags(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    tags: &Canonical<Tags<'_>>,
) -> RepoResult<()> {
    use crate::db::track_tag::{models::*, schema::*};
    let Tags {
        plain: plain_tags,
        facets,
    } = tags.as_ref();
    for plain_tag in plain_tags {
        let insertable = InsertableRecord::bind(track_id, None, plain_tag);
        insertable
            .insert_into(track_tag::table)
            .execute(db.as_mut())
            .map_err(repo_error)?;
    }
    for faceted_tags in facets {
        let FacetedTags { facet_id, tags } = faceted_tags;
        for tag in tags {
            let insertable = InsertableRecord::bind(track_id, Some(facet_id), tag);
            insertable
                .insert_into(track_tag::table)
                .execute(db.as_mut())
                .map_err(repo_error)?;
        }
    }
    Ok(())
}

fn update_track_tags(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    new_tags: &Canonical<Tags<'_>>,
) -> RepoResult<()> {
    let old_tags = load_track_tags(db, track_id)?;
    if &old_tags == new_tags {
        log::debug!("Keeping unmodified track tags");
        return Ok(());
    }
    delete_track_tags(db, track_id)?;
    insert_track_tags(db, track_id, new_tags)?;
    Ok(())
}

fn preload_entity(
    db: &mut crate::Connection<'_>,
    id: TrackId,
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

impl EntityRepo for Connection<'_> {
    fn resolve_track_id(&mut self, uid: &TrackUid) -> RepoResult<TrackId> {
        track::table
            .select(track::row_id)
            .filter(track::entity_uid.eq(EncodedEntityUid::from(uid).as_str()))
            .get_result::<RowId>(self.as_mut())
            .map_err(repo_error)
            .map(Into::into)
    }

    fn insert_track_entity(
        &mut self,
        media_source_id: MediaSourceId,
        created_entity: &TrackEntity,
    ) -> RepoResult<TrackId> {
        let insertable = InsertableRecord::bind(media_source_id, created_entity);
        let query = insertable.insert_into(track::table);
        let rows_affected = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert_eq!(1, rows_affected);
        let id = self.resolve_track_id(&created_entity.hdr.uid)?;
        insert_track_and_album_titles(
            self,
            id,
            created_entity.body.track.titles.as_canonical_slice(),
            created_entity.body.track.album.titles.as_canonical_slice(),
        )?;
        insert_track_and_album_actors(
            self,
            id,
            created_entity.body.track.actors.as_canonical_slice(),
            created_entity.body.track.album.actors.as_canonical_slice(),
        )?;
        insert_track_cues(
            self,
            id,
            created_entity.body.track.cues.as_canonical_slice(),
        )?;
        insert_track_tags(self, id, &created_entity.body.track.tags)?;
        Ok(id)
    }

    fn update_track_entity(
        &mut self,
        id: TrackId,
        media_source_id: MediaSourceId,
        updated_entity: &TrackEntity,
    ) -> RepoResult<()> {
        let record = UpdatableRecord::bind(
            updated_entity.hdr.rev,
            media_source_id,
            &updated_entity.body,
        );
        let target = track::table.filter(track::row_id.eq(RowId::from(id)));
        let query = diesel::update(target).set(&record);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        update_track_and_album_titles(
            self,
            id,
            updated_entity.body.track.titles.as_canonical_slice(),
            updated_entity.body.track.album.titles.as_canonical_slice(),
        )?;
        update_track_and_album_actors(
            self,
            id,
            updated_entity.body.track.actors.as_canonical_slice(),
            updated_entity.body.track.album.actors.as_canonical_slice(),
        )?;
        update_track_cues(
            self,
            id,
            updated_entity.body.track.cues.as_canonical_slice(),
        )?;
        update_track_tags(self, id, &updated_entity.body.track.tags)?;
        Ok(())
    }

    fn load_track_entity(&mut self, id: TrackId) -> RepoResult<(RecordHeader, TrackEntity)> {
        let queryable = view_track_search::table
            .filter(view_track_search::row_id.eq(RowId::from(id)))
            .get_result::<SearchQueryableRecord>(self.as_mut())
            .map_err(repo_error)?;
        let (_, media_source) = self.load_media_source(queryable.media_source_id.into())?;
        let preload = preload_entity(self, id, media_source)?;
        load_repo_entity(preload, queryable)
    }

    fn load_track_entity_by_uid(
        &mut self,
        uid: &TrackUid,
    ) -> RepoResult<(RecordHeader, TrackEntity)> {
        let queryable = view_track_search::table
            .filter(view_track_search::entity_uid.eq(EncodedEntityUid::from(uid).as_str()))
            .get_result::<SearchQueryableRecord>(self.as_mut())
            .map_err(repo_error)?;
        let (_, media_source) = self.load_media_source(queryable.media_source_id.into())?;
        let preload = preload_entity(self, queryable.row_id.into(), media_source)?;
        load_repo_entity(preload, queryable)
    }

    fn purge_track_entity(&mut self, id: TrackId) -> RepoResult<()> {
        let target = track::table.filter(track::row_id.eq(RowId::from(id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}

impl CollectionRepo for crate::Connection<'_> {
    fn load_track_entity_by_media_source_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
    ) -> RepoResult<(MediaSourceId, RecordHeader, TrackEntity)> {
        let media_source_id_subselect = select_media_source_id_filtered_by_content_path_predicate(
            collection_id,
            StringPredicate::Equals(content_path.to_borrowed().into()),
        );
        let queryable = view_track_search::table
            .filter(view_track_search::media_source_id.eq_any(media_source_id_subselect))
            .get_result::<SearchQueryableRecord>(self.as_mut())
            .map_err(repo_error)?;
        let media_source_id = queryable.media_source_id.into();
        let (_, media_source) = self.load_media_source(media_source_id)?;
        let preload = preload_entity(self, queryable.row_id.into(), media_source)?;
        let (record_header, entity) = load_repo_entity(preload, queryable)?;
        Ok((media_source_id, record_header, entity))
    }

    fn resolve_track_entity_header_by_media_source_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
    ) -> RepoResult<(MediaSourceId, RecordHeader, TrackHeader)> {
        let media_source_id_subselect = select_media_source_id_filtered_by_content_path_predicate(
            collection_id,
            StringPredicate::Equals(content_path.to_borrowed().into()),
        );
        let queryable = view_track_search::table
            .filter(view_track_search::media_source_id.eq_any(media_source_id_subselect))
            .get_result::<SearchQueryableRecord>(self.as_mut())
            .map_err(repo_error)?;
        Ok(queryable.into())
    }

    fn replace_track_by_media_source_content_path(
        &mut self,
        collection_id: CollectionId,
        params: ReplaceParams,
        mut track: Track,
    ) -> RepoResult<ReplaceOutcome> {
        let ReplaceParams {
            mode,
            preserve_collected_at,
            update_last_synchronized_rev,
        } = params;
        let loaded = self
            .load_track_entity_by_media_source_content_path(
                collection_id,
                &track.media_source.content.link.path,
            )
            .optional()?;
        if let Some((media_source_id, record_header, entity)) = loaded {
            // Update existing entry
            let id = record_header.id;
            if mode == ReplaceMode::CreateOnly {
                return Ok(ReplaceOutcome::NotUpdated(media_source_id, id, track));
            }
            if entity.body.track == track
                && (!update_last_synchronized_rev
                    || entity.body.last_synchronized_rev == Some(entity.hdr.rev))
            {
                return Ok(ReplaceOutcome::Unchanged(media_source_id, id, entity));
            }
            let updated_at = UtcDateTimeMs::now();
            if preserve_collected_at {
                if track.media_source.collected_at != entity.body.track.media_source.collected_at {
                    log::debug!(
                        "Preserving collected_at = {preserved}, discarding {discarded}",
                        preserved = entity.body.track.media_source.collected_at,
                        discarded = track.media_source.collected_at
                    );
                }
                track.media_source.collected_at = entity.body.track.media_source.collected_at;
            }
            if track == entity.body.track {
                return Ok(ReplaceOutcome::Unchanged(media_source_id, id, entity));
            }
            log::trace!("original = {:?}", entity.body);
            log::trace!("updated = {track:?}");
            if track.media_source != entity.body.track.media_source {
                self.update_media_source(media_source_id, updated_at, &track.media_source)?;
            }
            let entity_hdr = entity
                .raw
                .hdr
                .next_rev()
                .ok_or_else(|| RepoError::Other(anyhow!("no next revision")))?;
            let last_synchronized_rev = if update_last_synchronized_rev {
                if track.media_source.content.link.rev.is_some() {
                    // Mark the track as synchronized with the media source
                    Some(entity_hdr.rev)
                } else {
                    // Reset the synchronized revision
                    None
                }
            } else {
                // Keep the current synchronized revision
                entity.raw.body.last_synchronized_rev
            };
            let entity_body = TrackBody {
                track,
                updated_at,
                last_synchronized_rev,
                content_url: None,
            };
            let entity = TrackEntity::new(entity_hdr, entity_body);
            self.update_track_entity(id, media_source_id, &entity)?;
            Ok(ReplaceOutcome::Updated(media_source_id, id, entity))
        } else {
            // Create new entry
            if mode == ReplaceMode::UpdateOnly {
                return Ok(ReplaceOutcome::NotCreated(track));
            }
            let created_at = UtcDateTimeMs::now();
            let media_source_id = self
                .insert_media_source(collection_id, created_at, &track.media_source)?
                .id;
            let entity_hdr = TrackHeader::initial_random();
            let last_synchronized_rev =
                if update_last_synchronized_rev && track.media_source.content.link.rev.is_some() {
                    // Mark the track as synchronized with the media source
                    Some(entity_hdr.rev)
                } else {
                    None
                };
            let entity_body = TrackBody {
                track,
                updated_at: created_at,
                last_synchronized_rev,
                content_url: None,
            };
            let entity = TrackEntity::new(entity_hdr, entity_body);
            let id = self.insert_track_entity(media_source_id, &entity)?;
            Ok(ReplaceOutcome::Created(media_source_id, id, entity))
        }
    }

    fn search_tracks(
        &mut self,
        collection_id: CollectionId,
        pagination: &Pagination,
        filter: Option<&Filter>,
        ordering: &[SortOrder],
        collector: &mut dyn ReservableRecordCollector<Header = RecordHeader, Record = TrackEntity>,
    ) -> RepoResult<usize> {
        let mut query = view_track_search::table
            .select(view_track_search::all_columns)
            // TODO: Filtering by collection_id from the view is SLOOWWWWWWW!?!
            //.filter(view_track_search::collection_id.eq(RowId::from(collection_id)))
            // Filtering the collection_id by subselect through media_source (with an index) is much faster.
            .filter(
                view_track_search::media_source_id.eq_any(
                    media_source::table
                        .select(media_source::row_id)
                        .filter(media_source::collection_id.eq(RowId::from(collection_id))),
                ),
            )
            .into_boxed();

        if let Some(filter) = filter {
            query = query.filter(filter.build_expression());
        }

        for sort_order in ordering {
            query = sort_order.apply_to_query(query);
        }
        // Finally order by PK to preserve the relative order of results
        // even if no sorting was requested.
        query = query.then_order_by(view_track_search::row_id);

        // Pagination
        //FIXME: Extract into generic function crate::util::apply_pagination()
        let (limit, offset) = pagination_to_limit_offset(pagination);
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        log::debug!(
            "Loading results of SQL search query: {debug_query}",
            debug_query = diesel::debug_query(&query)
        );
        let timed = Instant::now();
        let records = query
            .load::<SearchQueryableRecord>(self.as_mut())
            .map_err(repo_error)?;
        let count = records.len();
        log::debug!(
            "Executing search query returned {count} record(s) and took {elapsed_millis} ms",
            elapsed_millis = timed.elapsed().as_secs_f64() * 1000.0,
        );

        let timed = Instant::now();
        collector.reserve(count);
        for record in records {
            let media_source_id = record.media_source_id.into();
            let (_, media_source) = self.load_media_source(media_source_id)?;
            let preload = preload_entity(self, record.row_id.into(), media_source)?;
            let (record_header, entity) = load_repo_entity(preload, record)?;
            collector.collect(record_header, entity);
        }
        log::debug!(
            "Loading and collecting {count} track(s) from database took {elapsed_millis} ms",
            elapsed_millis = timed.elapsed().as_secs_f64() * 1000.0,
        );

        Ok(count)
    }

    fn count_tracks(&mut self, collection_id: CollectionId) -> RepoResult<u64> {
        track::table
            .filter(track::media_source_id.eq_any(
                select_media_source_id_filtered_by_collection_id(collection_id),
            ))
            .count()
            .get_result::<i64>(self.as_mut())
            .map_err(repo_error)
            .map(|count| {
                debug_assert!(count >= 0);
                count.cast_unsigned()
            })
    }

    fn purge_tracks_by_media_source_content_path_predicate(
        &mut self,
        collection_id: CollectionId,
        content_path_predicate: StringPredicate<'_>,
    ) -> RepoResult<usize> {
        let media_source_id_subselect = select_media_source_id_filtered_by_content_path_predicate(
            collection_id,
            content_path_predicate,
        );
        let target = track::table.filter(track::media_source_id.eq_any(media_source_id_subselect));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        Ok(rows_affected)
    }

    fn find_unsynchronized_tracks(
        &mut self,
        collection_id: CollectionId,
        pagination: &Pagination,
        content_path_predicate: Option<StringPredicate<'_>>,
    ) -> RepoResult<Vec<(TrackHeader, RecordHeader, RecordTrail)>> {
        let mut query = collection::table
            .inner_join(media_source::table.inner_join(track::table))
            .select((
                collection::row_id,
                media_source::row_id,
                media_source::content_link_path,
                media_source::content_link_rev,
                track::row_id,
                track::row_created_ms,
                track::row_updated_ms,
                track::entity_uid,
                track::entity_rev,
                track::last_synchronized_rev,
            ))
            .filter(
                media_source::content_link_rev
                    .is_null()
                    .or(track::last_synchronized_rev.is_null())
                    .or(track::last_synchronized_rev.ne(track::entity_rev.nullable())),
            )
            .into_boxed();
        if let Some(content_path_predicate) = content_path_predicate {
            let media_source_id_subselect =
                select_media_source_id_filtered_by_content_path_predicate(
                    collection_id,
                    content_path_predicate,
                );
            // The optimizer will hopefully be able to inline this subselect that
            // allows to reuse the filtered select statement!
            query = query.filter(media_source::row_id.eq_any(media_source_id_subselect));
        }

        // Pagination
        //FIXME: Extract into generic function crate::util::apply_pagination()
        let (limit, offset) = pagination_to_limit_offset(pagination);
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let rows = query
            .load_iter::<(
                RowId,
                RowId,
                String,
                Option<i64>,
                RowId,
                i64,
                i64,
                String,
                i64,
                Option<i64>,
            ), _>(self.as_mut())
            .map_err(repo_error)?;

        rows.map(|row| {
            row.map_err(repo_error).map(
                |(
                    collection_id,
                    media_source_id,
                    content_link_path,
                    content_link_rev,
                    row_id,
                    row_created_ms,
                    row_updated_ms,
                    entity_uid,
                    entity_rev,
                    last_synchronized_rev,
                )| {
                    let record_header = RecordHeader {
                        id: row_id.into(),
                        created_at: UtcDateTimeMs::from_unix_timestamp_millis(row_created_ms),
                        updated_at: UtcDateTimeMs::from_unix_timestamp_millis(row_updated_ms),
                    };
                    let entity_header =
                        TrackHeader::from_untyped(decode_entity_header(&entity_uid, entity_rev));
                    let content_link = ContentLink {
                        path: content_link_path.into(),
                        rev: content_link_rev.map(ContentRevision::from_signed_value),
                    };
                    let last_synchronized_rev = last_synchronized_rev.map(decode_entity_revision);
                    let record_trail = RecordTrail {
                        collection_id: collection_id.into(),
                        media_source_id: media_source_id.into(),
                        content_link,
                        last_synchronized_rev,
                    };
                    (entity_header, record_header, record_trail)
                },
            )
        })
        .collect::<RepoResult<_>>()
    }
}

impl ActorRepo for crate::Connection<'_> {
    fn load_all_actor_names(
        &mut self,
        collection_id: Option<CollectionId>,
        summary_splitter: &ActorNamesSummarySplitter,
    ) -> RepoResult<Vec<String>> {
        use crate::db::{
            media_source::schema::*,
            track::schema::*,
            track_actor::{schema::*, *},
        };

        let mut query = track_actor::table
            .select((track_actor::name, track_actor::kind))
            .filter(track_actor::kind.ne(encode_kind(ActorKind::Sorting)))
            .into_boxed();
        if let Some(collection_id) = collection_id {
            query = query.filter(
                track_actor::track_id.eq_any(
                    track::table.select(track::row_id).filter(
                        track::media_source_id.eq_any(
                            media_source::table
                                .select(media_source::row_id)
                                .filter(media_source::collection_id.eq(RowId::from(collection_id))),
                        ),
                    ),
                ),
            );
        }

        let rows = query
            .load_iter::<(String, i16), _>(self.as_mut())
            .map_err(repo_error)?;

        let mut actor_names = HashSet::new();
        for row in rows {
            let (name, kind) = row.map_err(repo_error)?;
            let kind = decode_kind(kind).unwrap();
            match kind {
                ActorKind::Individual => {
                    actor_names.insert(name);
                }
                ActorKind::Summary => {
                    for name in summary_splitter.split_all(&name) {
                        if !actor_names.contains(name) {
                            actor_names.insert(name.to_owned());
                        }
                    }
                }
                ActorKind::Sorting => unreachable!(),
            }
        }

        Ok(actor_names.into_iter().collect())
    }
}
