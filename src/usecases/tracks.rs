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

mod _core {
    pub use aoide_core::track::Track;
}

use aoide_repo::{
    entity::{EntityBodyData, EntityData, EntityDataFormat, EntityDataVersion},
    tag::{
        AvgScoreCount as TagAvgScoreCount, CountParams as TagCountParams,
        FacetCount as TagFacetCount, FacetCountParams as TagFacetCountParams,
    },
    track::{
        CountTracksByAlbumParams, AlbumCountResults, Albums as _, LocateParams, ReplaceMode, ReplaceResult,
        Repo as _, SearchParams, Tags as _,
    },
    util::{UriPredicate, UriRelocation},
    Pagination, RepoResult, StringPredicate,
};

use aoide_core::{
    entity::{EntityHeader, EntityRevision, EntityUid},
    track::Entity,
};

use aoide_repo_sqlite::track::Repository;

use aoide_core_serde::track::Track;

use futures::future::{self, Future};

///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct TrackReplacement {
    // The URI for locating any existing track that is supposed
    // to replaced by the provided track.
    pub media_uri: String,

    pub track: Track,
}

#[derive(Clone, Debug, Default)]
pub struct ReplacedTracks {
    pub created: Vec<EntityHeader>,
    pub updated: Vec<EntityHeader>,
    pub skipped: Vec<EntityHeader>,
    pub rejected: Vec<String>,  // e.g. ambiguous or inconsistent
    pub discarded: Vec<String>, // e.g. nonexistent and need to be created
}

const ENTITY_DATA_FORMAT: EntityDataFormat = EntityDataFormat::JSON;

const ENTITY_DATA_VERSION: EntityDataVersion = EntityDataVersion { major: 0, minor: 0 };

pub fn write_json_body_data(track: &Track) -> Fallible<EntityBodyData> {
    Ok((
        ENTITY_DATA_FORMAT,
        ENTITY_DATA_VERSION,
        serde_json::to_vec(track)?,
    ))
}

fn read_json_entity(entity_data: EntityData) -> Fallible<Entity> {
    let (hdr, json_data) = load_json_entity_data(entity_data)?;
    let track: Track = serde_json::from_slice(&json_data)?;
    Ok(Entity::new(hdr, _core::Track::from(track)))
}

pub fn load_json_entity_data(entity_data: EntityData) -> Fallible<(EntityHeader, Vec<u8>)> {
    let (hdr, (data_fmt, data_ver, json_data)) = entity_data;
    if data_fmt != ENTITY_DATA_FORMAT {
        let e = failure::format_err!(
            "Unsupported data format when loading track {}: expected = {:?}, actual = {:?}",
            hdr.uid,
            ENTITY_DATA_FORMAT,
            data_fmt
        );
        return Err(e);
    }
    if data_ver < ENTITY_DATA_VERSION {
        // TODO: Data migration from an older version
        unimplemented!();
    }
    if data_ver == ENTITY_DATA_VERSION {
        return Ok((hdr, json_data));
    }
    let e = failure::format_err!(
        "Unsupported data version when loading track {}: expected = {:?}, actual = {:?}",
        hdr.uid,
        ENTITY_DATA_VERSION,
        data_ver
    );
    Err(e)
}

pub fn create_track(
    db: &SqlitePooledConnection,
    new_track: _core::Track,
    body_data: EntityBodyData,
) -> RepoResult<EntityHeader> {
    let repository = Repository::new(&*db);
    let hdr = EntityHeader::initial_random();
    let entity = Entity::new(hdr.clone(), new_track);
    db.transaction::<_, Error, _>(|| repository.insert_track(entity, body_data).map(|()| hdr))
}

pub fn update_track(
    db: &SqlitePooledConnection,
    track: Entity,
    body_data: EntityBodyData,
) -> RepoResult<(EntityRevision, Option<EntityRevision>)> {
    let repository = Repository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.update_track(track, body_data))
}

pub fn delete_track(db: &SqlitePooledConnection, uid: &EntityUid) -> RepoResult<Option<()>> {
    let repository = Repository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.delete_track(uid))
}

pub fn load_track(
    pooled_connection: &SqlitePooledConnection,
    uid: &EntityUid,
) -> RepoResult<Option<EntityData>> {
    let repository = Repository::new(&*pooled_connection);
    pooled_connection.transaction::<_, Error, _>(|| repository.load_track(uid))
}

pub fn list_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
) -> impl Future<Item = Vec<EntityData>, Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.search_tracks(collection_uid.as_ref(), pagination, Default::default())
    }))
}

pub fn search_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
    params: SearchParams,
) -> impl Future<Item = Vec<EntityData>, Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.search_tracks(collection_uid.as_ref(), pagination, params)
    }))
}

pub fn locate_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
    params: LocateParams,
) -> impl Future<Item = Vec<EntityData>, Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.locate_tracks(collection_uid.as_ref(), pagination, params)
    }))
}

pub fn replace_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    mode: ReplaceMode,
    replacements: impl Iterator<Item = TrackReplacement>,
) -> impl Future<Item = ReplacedTracks, Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        let mut results = ReplacedTracks::default();
        for replacement in replacements {
            let body_data = write_json_body_data(&replacement.track)?;
            let (data_fmt, data_ver, _) = body_data;
            let media_uri = replacement.media_uri;
            let replace_result = repository.replace_track(
                collection_uid.as_ref(),
                media_uri.clone(),
                mode,
                replacement.track.into(),
                body_data,
            )?;
            use ReplaceResult::*;
            match replace_result {
                AmbiguousMediaUri(count) => {
                    log::warn!(
                        "Cannot replace track with ambiguous media URI '{}' that matches {} tracks",
                        media_uri,
                        count
                    );
                    results.rejected.push(media_uri);
                }
                IncompatibleFormat(fmt) => {
                    log::warn!(
                        "Incompatible data formats for track with media URI '{}': Current = {}, replacement = {}",
                        media_uri,
                        fmt,
                        data_fmt
                    );
                    results.rejected.push(media_uri);
                }
                IncompatibleVersion(ver) => {
                    log::warn!(
                        "Incompatible data versions for track with media URI '{}': Current = {}, replacement = {}",
                        media_uri,
                        ver,
                        data_ver
                    );
                    results.rejected.push(media_uri);
                }
                NotCreated => {
                    results.discarded.push(media_uri);
                }
                Unchanged(hdr) => {
                    results.skipped.push(hdr);
                }
                Created(hdr) => {
                    results.created.push(hdr);
                }
                Updated(hdr) => {
                    results.updated.push(hdr);
                }
            }
        }
        Ok(results)
    }))
}

pub fn purge_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    uri_predicates: impl IntoIterator<Item = UriPredicate>,
) -> impl Future<Item = (), Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        for uri_predicate in uri_predicates {
            use StringPredicate::*;
            use UriPredicate::*;
            let locate_params = match &uri_predicate {
                Prefix(media_uri) => LocateParams {
                    media_uri: StartsWith(media_uri.to_owned()),
                },
                Exact(media_uri) => LocateParams {
                    media_uri: Equals(media_uri.to_owned()),
                },
            };
            let entities = repository.locate_tracks(
                collection_uid.as_ref(),
                Default::default(),
                locate_params,
            )?;
            log::debug!(
                "Found {} track(s) that match {:?} as candidates for purging",
                entities.len(),
                uri_predicate,
            );
            for entity in entities.into_iter() {
                let Entity { hdr, mut body, .. } = read_json_entity(entity)?;
                let purged = match &uri_predicate {
                    Prefix(ref uri_prefix) => body.purge_media_source_by_uri_prefix(uri_prefix),
                    Exact(ref uri) => body.purge_media_source_by_uri(uri),
                };
                if purged > 0 {
                    if body.media_sources.is_empty() {
                        log::debug!(
                            "Deleting track {} after purging all (= {}) media sources",
                            hdr.uid,
                            purged,
                        );
                        repository.delete_track(&hdr.uid)?;
                    } else {
                        log::debug!(
                            "Updating track {} after purging {} of {} media source(s)",
                            hdr.uid,
                            purged,
                            purged + body.media_sources.len(),
                        );
                        // TODO: Avoid temporary clone
                        let json_data = write_json_body_data(&body.clone().into())?;
                        let entity = Entity::new(hdr, body);
                        let updated = repository.update_track(entity, json_data)?;
                        debug_assert!(updated.1.is_some());
                    }
                } else {
                    log::debug!("No media sources purged from track {}", hdr.uid);
                }
            }
        }
        Ok(())
    }))
}

pub fn relocate_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    uri_relocations: impl IntoIterator<Item = UriRelocation>,
) -> impl Future<Item = (), Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        for uri_relocation in uri_relocations {
            let locate_params = match &uri_relocation.predicate {
                UriPredicate::Prefix(uri_prefix) => LocateParams {
                    media_uri: StringPredicate::StartsWith(uri_prefix.to_owned()),
                },
                UriPredicate::Exact(uri) => LocateParams {
                    media_uri: StringPredicate::Equals(uri.to_owned()),
                },
            };
            let tracks = repository.locate_tracks(
                collection_uid.as_ref(),
                Default::default(),
                locate_params,
            )?;
            log::debug!(
                "Found {} track(s) that match {:?} as candidates for relocating",
                tracks.len(),
                uri_relocation.predicate,
            );
            for entity_data in tracks {
                let (hdr, json_data) = load_json_entity_data(entity_data)?;
                let mut track = _core::Track::from(serde_json::from_slice::<Track>(&json_data)?);
                let relocated = match &uri_relocation.predicate {
                    UriPredicate::Prefix(uri_prefix) => track.relocate_media_source_by_uri_prefix(
                        &uri_prefix,
                        &uri_relocation.replacement,
                    ),
                    UriPredicate::Exact(uri) => {
                        track.relocate_media_source_by_uri(&uri, &uri_relocation.replacement)
                    }
                };
                if relocated > 0 {
                    log::debug!(
                        "Updating track {} after relocating {} source(s)",
                        hdr.uid,
                        relocated,
                    );
                    // TODO: Avoid temporary clone
                    let json_data = write_json_body_data(&track.clone().into())?;
                    let entity = Entity::new(hdr, track);
                    let updated = repository.update_track(entity, json_data)?;
                    debug_assert!(updated.1.is_some());
                } else {
                    log::debug!("No sources relocated for track {}", hdr.uid);
                }
            }
        }
        Ok(())
    }))
}

pub fn count_tracks_by_album(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
    params: &CountTracksByAlbumParams,
) -> impl Future<Item = Vec<AlbumCountResults>, Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_tracks_by_album(collection_uid.as_ref(), params, pagination)
    }))
}

pub fn count_tracks_by_tag(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
    mut params: TagCountParams,
) -> impl Future<Item = Vec<TagAvgScoreCount>, Error = Error> {
    params.dedup_facets();
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_tracks_by_tag(collection_uid.as_ref(), &params, pagination)
    }))
}

pub fn count_tracks_by_tag_facet(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
    mut params: TagFacetCountParams,
) -> impl Future<Item = Vec<TagFacetCount>, Error = Error> {
    params.dedup_facets();
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_tracks_by_tag_facet(collection_uid.as_ref(), &params, pagination)
    }))
}
