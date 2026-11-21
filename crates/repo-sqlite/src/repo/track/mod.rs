// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    collections::{BTreeMap, HashSet},
    time::Instant,
};

use anyhow::anyhow;
use diesel::prelude::*;
use nonicle::{Canonical, CanonicalizeInto as _};

use aoide_core::{
    Track, TrackBody, TrackEntity, TrackHeader, TrackUid,
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
    filter::StringPredicate,
    track::search::{Filter, Scope, SortOrder},
};
use aoide_repo::{
    CollectionId, MediaSourceId, RepoError, RepoResult, ReservableRecordCollector, TrackId,
    media::source::{CollectionRepo as _, Repo as _},
    track::{
        ActorRepo, CollectionRepo, EntityRepo, LoadedTrack, RecordHeader, RecordTrail, ReplaceMode,
        ReplaceOutcome, ReplaceParams, TagRepo,
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
        apply_pagination,
        entity::{decode_entity_header, decode_entity_revision, encode_entity_uid},
    },
};

mod search;
use self::search::{apply_sort_order, build_filter_expression};

/// Maximum number of tracks whose children are loaded with a single query.
///
/// Limits the number of bound SQL parameters.
const PRELOAD_TRACKS_CHUNK_SIZE: usize = 500;

/// The children of a track that are stored in separate tables.
#[derive(Debug, Default)]
struct TrackChildren {
    track_titles: Vec<Title>,
    album_titles: Vec<Title>,
    track_actors: Vec<Actor>,
    album_actors: Vec<Actor>,
    cues: Vec<Cue>,
    tags: Tags<'static>,
}

type TrackChildrenById = BTreeMap<TrackId, TrackChildren>;

fn track_row_ids(ids: &[TrackId]) -> Vec<RowId> {
    ids.iter().copied().map(RowId::from).collect()
}

fn load_titles_into(
    db: &mut Connection<'_>,
    ids: &[TrackId],
    children: &mut TrackChildrenById,
) -> RepoResult<()> {
    use crate::db::track_title::{models::*, schema::*, *};

    let query = track_title::table
        .filter(track_title::track_id.eq_any(track_row_ids(ids)))
        // Establish canonical ordering on load!
        .order_by((
            track_title::track_id,
            track_title::scope,
            track_title::kind,
            track_title::name,
        ));
    let rows = query
        .load_iter::<QueryableRecord, _>(db.as_mut())
        .map_err(repo_error)?;
    for row in rows {
        let (_, record) = row
            .map_err(repo_error)?
            .try_into()
            .map_err(RepoError::Other)?;
        let Record {
            track_id,
            scope,
            title,
        } = record;
        let children = children.entry(track_id).or_default();
        match scope {
            Scope::Track => {
                children.track_titles.push(title);
            }
            Scope::Album => {
                children.album_titles.push(title);
            }
        }
    }
    Ok(())
}

// TODO: Define a dedicated return type
#[expect(clippy::type_complexity)]
fn load_track_and_album_titles(
    db: &mut Connection<'_>,
    id: TrackId,
) -> RepoResult<(Canonical<Vec<Title>>, Canonical<Vec<Title>>)> {
    let mut children = TrackChildrenById::new();
    load_titles_into(db, &[id], &mut children)?;
    let TrackChildren {
        track_titles,
        album_titles,
        ..
    } = children.remove(&id).unwrap_or_default();
    // The ordering returned by the SQL database might differ from the
    // canonical ordering due to custom text column collations!
    Ok((
        track_titles.canonicalize_into(),
        album_titles.canonicalize_into(),
    ))
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

/// The old titles are loaded if `old_track` is unknown.
fn update_track_and_album_titles(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    old_track: Option<&Track>,
    new_track_titles: Canonical<&[Title]>,
    new_album_titles: Canonical<&[Title]>,
) -> RepoResult<()> {
    let loaded;
    let old_titles = if let Some(old_track) = old_track {
        (
            old_track.titles.as_canonical_slice(),
            old_track.album.titles.as_canonical_slice(),
        )
    } else {
        loaded = load_track_and_album_titles(db, track_id)?;
        (loaded.0.as_canonical_slice(), loaded.1.as_canonical_slice())
    };
    if old_titles == (new_track_titles, new_album_titles) {
        log::debug!("Keeping unmodified track/album titles");
        return Ok(());
    }
    delete_track_and_album_titles(db, track_id)?;
    insert_track_and_album_titles(db, track_id, new_track_titles, new_album_titles)?;
    Ok(())
}

fn load_actors_into(
    db: &mut Connection<'_>,
    ids: &[TrackId],
    children: &mut TrackChildrenById,
) -> RepoResult<()> {
    use crate::db::track_actor::{models::*, schema::*, *};

    let query = track_actor::table
        .filter(track_actor::track_id.eq_any(track_row_ids(ids)))
        // Establish canonical ordering on load!
        .order_by((
            track_actor::track_id,
            track_actor::scope,
            track_actor::role,
            track_actor::kind,
            track_actor::name,
        ));
    let rows = query
        .load_iter::<QueryableRecord, _>(db.as_mut())
        .map_err(repo_error)?;
    for row in rows {
        let (_, record) = row
            .map_err(repo_error)?
            .try_into()
            .map_err(RepoError::Other)?;
        let Record {
            track_id,
            scope,
            actor,
        } = record;
        let children = children.entry(track_id).or_default();
        match scope {
            Scope::Track => {
                children.track_actors.push(actor);
            }
            Scope::Album => {
                children.album_actors.push(actor);
            }
        }
    }
    Ok(())
}

// TODO: Define a dedicated return type
#[expect(clippy::type_complexity)]
fn load_track_and_album_actors(
    db: &mut Connection<'_>,
    id: TrackId,
) -> RepoResult<(Canonical<Vec<Actor>>, Canonical<Vec<Actor>>)> {
    let mut children = TrackChildrenById::new();
    load_actors_into(db, &[id], &mut children)?;
    let TrackChildren {
        track_actors,
        album_actors,
        ..
    } = children.remove(&id).unwrap_or_default();
    // The ordering returned by the SQL database might differ from the
    // canonical ordering due to custom text column collations!
    Ok((
        track_actors.canonicalize_into(),
        album_actors.canonicalize_into(),
    ))
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

/// The old actors are loaded if `old_track` is unknown.
fn update_track_and_album_actors(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    old_track: Option<&Track>,
    new_track_actors: Canonical<&[Actor]>,
    new_album_actors: Canonical<&[Actor]>,
) -> RepoResult<()> {
    let loaded;
    let old_actors = if let Some(old_track) = old_track {
        (
            old_track.actors.as_canonical_slice(),
            old_track.album.actors.as_canonical_slice(),
        )
    } else {
        loaded = load_track_and_album_actors(db, track_id)?;
        (loaded.0.as_canonical_slice(), loaded.1.as_canonical_slice())
    };
    if old_actors == (new_track_actors, new_album_actors) {
        log::debug!("Keeping unmodified track/album actors");
        return Ok(());
    }
    delete_track_and_album_actors(db, track_id)?;
    insert_track_and_album_actors(db, track_id, new_track_actors, new_album_actors)?;
    Ok(())
}

fn load_cues_into(
    db: &mut Connection<'_>,
    ids: &[TrackId],
    children: &mut TrackChildrenById,
) -> RepoResult<()> {
    use crate::db::track_cue::{models::*, schema::*, *};

    let query = track_cue::table
        .filter(track_cue::track_id.eq_any(track_row_ids(ids)))
        // Establish canonical ordering on load!
        .order_by((
            track_cue::track_id,
            track_cue::bank_idx,
            track_cue::slot_idx,
        ));
    let rows = query
        .load_iter::<QueryableRecord, _>(db.as_mut())
        .map_err(repo_error)?;
    for row in rows {
        let (_, Record { track_id, cue }) = row.map_err(repo_error)?.into();
        children.entry(track_id).or_default().cues.push(cue);
    }
    Ok(())
}

fn load_track_cues(db: &mut Connection<'_>, id: TrackId) -> RepoResult<Canonical<Vec<Cue>>> {
    let mut children = TrackChildrenById::new();
    load_cues_into(db, &[id], &mut children)?;
    let TrackChildren { cues, .. } = children.remove(&id).unwrap_or_default();
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

/// The old cues are loaded if `old_track` is unknown.
fn update_track_cues(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    old_track: Option<&Track>,
    new_cues: Canonical<&[Cue]>,
) -> RepoResult<()> {
    let loaded;
    let old_cues = if let Some(old_track) = old_track {
        old_track.cues.as_canonical_slice()
    } else {
        loaded = load_track_cues(db, track_id)?;
        loaded.as_canonical_slice()
    };
    if old_cues == new_cues {
        log::debug!("Keeping unmodified track cues");
        return Ok(());
    }
    delete_track_cues(db, track_id)?;
    insert_track_cues(db, track_id, new_cues)?;
    Ok(())
}

fn load_tags_into(
    db: &mut Connection<'_>,
    ids: &[TrackId],
    children: &mut TrackChildrenById,
) -> RepoResult<()> {
    use crate::db::track_tag::{models::*, schema::*};

    let query = track_tag::table
        .filter(track_tag::track_id.eq_any(track_row_ids(ids)))
        // Establish canonical ordering on load!
        .order_by((
            track_tag::track_id,
            track_tag::facet,
            track_tag::label,
            track_tag::score.desc(),
        ));
    let rows = query
        .load_iter::<QueryableRecord, _>(db.as_mut())
        .map_err(repo_error)?;
    for row in rows {
        let (_, record) = row.map_err(repo_error)?.into();
        let tags = &mut children.entry(record.track_id).or_default().tags;
        let (facet_id, tag) = record.into();
        if let Some(facet_id) = facet_id {
            if let Some(faceted_tags) = tags.facets.last_mut()
                && faceted_tags.facet_id == facet_id
            {
                faceted_tags.tags.push(tag);
                continue;
            }
            tags.facets.push(FacetedTags {
                facet_id,
                tags: vec![tag],
            });
        } else {
            tags.plain.push(tag);
        }
    }
    Ok(())
}

fn load_track_tags(db: &mut Connection<'_>, id: TrackId) -> RepoResult<Canonical<Tags<'static>>> {
    let mut children = TrackChildrenById::new();
    load_tags_into(db, &[id], &mut children)?;
    let TrackChildren { tags, .. } = children.remove(&id).unwrap_or_default();
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

/// The old tags are loaded if `old_track` is unknown.
fn update_track_tags(
    db: &mut crate::Connection<'_>,
    track_id: TrackId,
    old_track: Option<&Track>,
    new_tags: &Canonical<Tags<'_>>,
) -> RepoResult<()> {
    let loaded;
    let old_tags = if let Some(old_track) = old_track {
        &old_track.tags
    } else {
        loaded = load_track_tags(db, track_id)?;
        &loaded
    };
    if old_tags == new_tags {
        log::debug!("Keeping unmodified track tags");
        return Ok(());
    }
    delete_track_tags(db, track_id)?;
    insert_track_tags(db, track_id, new_tags)?;
    Ok(())
}

fn load_media_sources(
    db: &mut Connection<'_>,
    ids: &[MediaSourceId],
) -> RepoResult<BTreeMap<MediaSourceId, Source>> {
    use crate::db::media_source::models::QueryableRecord;

    let rows = media_source::table
        .filter(media_source::row_id.eq_any(ids.iter().copied().map(RowId::from)))
        .load_iter::<QueryableRecord, _>(db.as_mut())
        .map_err(repo_error)?;
    rows.map(|row| {
        let (header, media_source) = row
            .map_err(repo_error)?
            .try_into()
            .map_err(RepoError::Other)?;
        Ok((header.id, media_source))
    })
    .collect()
}

/// Preload the media sources and children of multiple tracks.
///
/// Issues a fixed number of queries independent of the number of tracks.
/// The results are returned in the order of `ids`.
fn preload_entities(
    db: &mut Connection<'_>,
    ids: &[(TrackId, MediaSourceId)],
) -> RepoResult<Vec<EntityPreload>> {
    let track_ids = ids.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    let media_source_ids = ids.iter().map(|(_, id)| *id).collect::<Vec<_>>();
    let mut media_sources = load_media_sources(db, &media_source_ids)?;
    let mut children = TrackChildrenById::new();
    load_titles_into(db, &track_ids, &mut children)?;
    load_actors_into(db, &track_ids, &mut children)?;
    load_cues_into(db, &track_ids, &mut children)?;
    load_tags_into(db, &track_ids, &mut children)?;
    ids.iter()
        .map(|(id, media_source_id)| {
            let media_source = media_sources
                .remove(media_source_id)
                .ok_or(RepoError::NotFound)?;
            let TrackChildren {
                track_titles,
                album_titles,
                track_actors,
                album_actors,
                cues,
                tags,
            } = children.remove(id).unwrap_or_default();
            // The ordering returned by the SQL database might differ from the
            // canonical ordering due to custom text column collations!
            Ok(EntityPreload {
                media_source,
                track_titles: track_titles.canonicalize_into(),
                track_actors: track_actors.canonicalize_into(),
                album_titles: album_titles.canonicalize_into(),
                album_actors: album_actors.canonicalize_into(),
                tags: tags.canonicalize_into(),
                cues: Canonical::tie(cues),
            })
        })
        .collect()
}

fn preload_entity(
    db: &mut Connection<'_>,
    id: TrackId,
    media_source_id: MediaSourceId,
) -> RepoResult<EntityPreload> {
    let mut preloads = preload_entities(db, &[(id, media_source_id)])?;
    debug_assert_eq!(preloads.len(), 1);
    preloads.pop().ok_or(RepoError::NotFound)
}

/// Update a track and its children.
///
/// If the current state of the track is known then the children are
/// compared with it instead of loading them.
fn update_track_entity(
    db: &mut Connection<'_>,
    id: TrackId,
    media_source_id: MediaSourceId,
    updated_entity: &TrackEntity,
    old_track: Option<&Track>,
) -> RepoResult<()> {
    let record = UpdatableRecord::bind(
        updated_entity.hdr.rev,
        media_source_id,
        &updated_entity.body,
    );
    let target = track::table.filter(track::row_id.eq(RowId::from(id)));
    let query = diesel::update(target).set(&record);
    let rows_affected: usize = query.execute(db.as_mut()).map_err(repo_error)?;
    debug_assert!(rows_affected <= 1);
    if rows_affected < 1 {
        return Err(RepoError::NotFound);
    }
    let new_track = &updated_entity.body.track;
    update_track_and_album_titles(
        db,
        id,
        old_track,
        new_track.titles.as_canonical_slice(),
        new_track.album.titles.as_canonical_slice(),
    )?;
    update_track_and_album_actors(
        db,
        id,
        old_track,
        new_track.actors.as_canonical_slice(),
        new_track.album.actors.as_canonical_slice(),
    )?;
    update_track_cues(db, id, old_track, new_track.cues.as_canonical_slice())?;
    update_track_tags(db, id, old_track, &new_track.tags)?;
    Ok(())
}

impl EntityRepo for Connection<'_> {
    fn resolve_track_id(&mut self, uid: &TrackUid) -> RepoResult<TrackId> {
        track::table
            .select(track::row_id)
            .filter(track::entity_uid.eq(encode_entity_uid(uid).as_str()))
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
        let id = insertable
            .insert_into(track::table)
            .returning(track::row_id)
            .get_result::<RowId>(self.as_mut())
            .map_err(repo_error)?
            .into();
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
        update_track_entity(self, id, media_source_id, updated_entity, None)
    }

    fn load_track_entity(&mut self, id: TrackId) -> RepoResult<(RecordHeader, TrackEntity)> {
        let queryable = view_track_search::table
            .filter(view_track_search::row_id.eq(RowId::from(id)))
            .get_result::<SearchQueryableRecord>(self.as_mut())
            .map_err(repo_error)?;
        let preload = preload_entity(self, id, queryable.media_source_id.into())?;
        load_repo_entity(preload, queryable)
    }

    fn load_track_entity_by_uid(
        &mut self,
        uid: &TrackUid,
    ) -> RepoResult<(RecordHeader, TrackEntity)> {
        let queryable = view_track_search::table
            .filter(view_track_search::entity_uid.eq(encode_entity_uid(uid).as_str()))
            .get_result::<SearchQueryableRecord>(self.as_mut())
            .map_err(repo_error)?;
        let preload = preload_entity(
            self,
            queryable.row_id.into(),
            queryable.media_source_id.into(),
        )?;
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

const RECORD_TRAIL_COLUMNS: (
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
) = (
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
);

type RecordTrailRow = (
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
);

fn decode_record_trail_row(row: RecordTrailRow) -> (TrackHeader, RecordHeader, RecordTrail) {
    let (
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
    ) = row;
    let record_header = RecordHeader {
        id: row_id.into(),
        created_at: UtcDateTimeMs::from_unix_timestamp_millis(row_created_ms),
        updated_at: UtcDateTimeMs::from_unix_timestamp_millis(row_updated_ms),
    };
    let entity_header = TrackHeader::from_untyped(decode_entity_header(&entity_uid, entity_rev));
    let content_link = ContentLink {
        path: content_link_path.into(),
        rev: content_link_rev.map(ContentRevision::from_signed_value),
    };
    let record_trail = RecordTrail {
        collection_id: collection_id.into(),
        media_source_id: media_source_id.into(),
        content_link,
        last_synchronized_rev: last_synchronized_rev.map(decode_entity_revision),
    };
    (entity_header, record_header, record_trail)
}

impl CollectionRepo for crate::Connection<'_> {
    fn load_track_entity_by_media_source_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
    ) -> RepoResult<(MediaSourceId, RecordHeader, TrackEntity)> {
        let media_source_id_subselect = select_media_source_id_filtered_by_content_path_predicate(
            collection_id,
            &StringPredicate::Equals(content_path.to_borrowed().into()),
        );
        let queryable = view_track_search::table
            .filter(view_track_search::media_source_id.eq_any(media_source_id_subselect))
            .get_result::<SearchQueryableRecord>(self.as_mut())
            .map_err(repo_error)?;
        let media_source_id = queryable.media_source_id.into();
        let preload = preload_entity(self, queryable.row_id.into(), media_source_id)?;
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
            &StringPredicate::Equals(content_path.to_borrowed().into()),
        );
        let queryable = view_track_search::table
            .filter(view_track_search::media_source_id.eq_any(media_source_id_subselect))
            .get_result::<SearchQueryableRecord>(self.as_mut())
            .map_err(repo_error)?;
        Ok(queryable.into())
    }

    fn resolve_track_record_trail_by_media_source_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &ContentPath<'_>,
    ) -> RepoResult<(TrackHeader, RecordHeader, RecordTrail)> {
        let media_source_id_subselect = select_media_source_id_filtered_by_content_path_predicate(
            collection_id,
            &StringPredicate::Equals(content_path.to_borrowed().into()),
        );
        collection::table
            .inner_join(media_source::table.inner_join(track::table))
            .select(RECORD_TRAIL_COLUMNS)
            .filter(media_source::row_id.eq_any(media_source_id_subselect))
            .get_result::<RecordTrailRow>(self.as_mut())
            .map(decode_record_trail_row)
            .map_err(repo_error)
    }

    fn replace_loaded_track(
        &mut self,
        collection_id: CollectionId,
        params: ReplaceParams,
        mut track: Track,
        loaded: Option<LoadedTrack>,
    ) -> RepoResult<ReplaceOutcome> {
        let ReplaceParams {
            mode,
            preserve_collected_at,
            update_last_synchronized_rev,
        } = params;
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
            if track == entity.body.track
                && (!update_last_synchronized_rev
                    || entity.body.last_synchronized_rev == Some(entity.hdr.rev))
            {
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
            let old_track = entity.raw.body.track;
            let entity_body = TrackBody {
                track,
                updated_at,
                last_synchronized_rev,
                content_url: None,
            };
            let entity = TrackEntity::new(entity_hdr, entity_body);
            update_track_entity(self, id, media_source_id, &entity, Some(&old_track))?;
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
        order: &[SortOrder],
        collector: &mut dyn ReservableRecordCollector<Header = RecordHeader, Record = TrackEntity>,
    ) -> RepoResult<usize> {
        let mut query = view_track_search::table
            .select(view_track_search::all_columns)
            // TODO: Filtering by collection_id from the view is SLOOWWWWWWW!?!
            //.filter(view_track_search::collection_id.eq(RowId::from(collection_id)))
            // Filtering the collection_id by subselect through media_source (with an index) is much faster.
            // Adding zero prevents using the column for an index lookup. Otherwise SQLite might combine
            // it with other `row_id IN (...)` filters for a lookup by (media_source_id, row_id) and probe
            // all pairs of both lists, i.e. all tracks of the collection times all candidates of the filter.
            .filter(
                (view_track_search::media_source_id + 0_i64).eq_any(
                    media_source::table
                        .select(media_source::row_id)
                        .filter(media_source::collection_id.eq(RowId::from(collection_id))),
                ),
            )
            .into_boxed();

        if let Some(filter) = filter {
            query = query.filter(build_filter_expression(filter));
        }

        for sort_order in order {
            query = apply_sort_order(*sort_order, query);
        }
        // Finally order by PK to preserve the relative order of results
        // even if no sorting was requested.
        query = query.then_order_by(view_track_search::row_id);

        // Pagination
        query = apply_pagination(query, pagination);

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
        let mut records = records.into_iter();
        loop {
            let chunk = records
                .by_ref()
                .take(PRELOAD_TRACKS_CHUNK_SIZE)
                .collect::<Vec<_>>();
            if chunk.is_empty() {
                break;
            }
            let ids = chunk
                .iter()
                .map(|record| (record.row_id.into(), record.media_source_id.into()))
                .collect::<Vec<_>>();
            let preloads = preload_entities(self, &ids)?;
            for (record, preload) in chunk.into_iter().zip(preloads) {
                let (record_header, entity) = load_repo_entity(preload, record)?;
                collector.collect(record_header, entity);
            }
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
            &content_path_predicate,
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
            .select(RECORD_TRAIL_COLUMNS)
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
                    &content_path_predicate,
                );
            // The optimizer will hopefully be able to inline this subselect that
            // allows to reuse the filtered select statement!
            query = query.filter(media_source::row_id.eq_any(media_source_id_subselect));
        }

        // Pagination
        query = apply_pagination(query, pagination);

        let rows = query
            .load_iter::<RecordTrailRow, _>(self.as_mut())
            .map_err(repo_error)?;
        rows.map(|row| row.map_err(repo_error).map(decode_record_trail_row))
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

impl TagRepo for crate::Connection<'_> {
    fn count_track_tag_labels(
        &mut self,
        collection_id: Option<CollectionId>,
        facet_id: Option<&FacetId>,
    ) -> RepoResult<Vec<(Label<'static>, usize)>> {
        use crate::db::{media_source::schema::*, track::schema::*, track_tag::schema::*};

        let mut query = track_tag::table
            .inner_join(track::table)
            .group_by(track_tag::label)
            .select((track_tag::label, diesel::dsl::count_star()))
            .filter(track_tag::label.is_not_null())
            .order_by(track_tag::label)
            .into_boxed();
        query = if let Some(facet_id) = facet_id {
            query.filter(track_tag::facet.eq(facet_id.as_str()))
        } else {
            query.filter(track_tag::facet.is_null())
        };
        if let Some(collection_id) = collection_id {
            query = query.filter(
                track::media_source_id.eq_any(
                    media_source::table
                        .select(media_source::row_id)
                        .filter(media_source::collection_id.eq(RowId::from(collection_id))),
                ),
            );
        }
        let rows = query
            .load::<(Option<String>, i64)>(self.as_mut())
            .map_err(repo_error)?;
        // Each track has at most one tag per facet and label.
        let label_counts = rows
            .into_iter()
            .filter_map(|(label, count)| {
                let label = Label::from_unchecked(label?);
                debug_assert!(semval::IsValid::is_valid(&label));
                Some((label, usize::try_from(count).unwrap_or(usize::MAX)))
            })
            .collect();
        Ok(label_counts)
    }
}
