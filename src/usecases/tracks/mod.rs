// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

pub mod json;

mod _serde {
    pub use aoide_core_serde::track::Track;
}

use aoide_core::{
    entity::{EntityHeader, EntityRevisionUpdateResult, EntityUid},
    track::{Entity, Track},
};

use aoide_repo::{
    entity::{EntityBodyData, EntityData},
    tag::{
        AvgScoreCount as TagAvgScoreCount, CountParams as TagCountParams,
        FacetCount as TagFacetCount, FacetCountParams as TagFacetCountParams,
    },
    track::{
        AlbumCountResults, Albums as _, CountTracksByAlbumParams, LocateParams, ReplaceMode,
        ReplaceResult, Repo as _, SearchParams, Tags as _,
    },
    util::{UriPredicate, UriRelocation},
    Pagination, RepoResult, StringPredicate,
};

use aoide_repo_sqlite::track::Repository;

///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct TrackReplacement {
    // The URI for locating any existing track that is supposed
    // to replaced by the provided track.
    pub media_uri: String,

    pub track: _serde::Track,
}

#[derive(Clone, Debug, Default)]
pub struct ReplacedTracks {
    pub created: Vec<EntityHeader>,
    pub updated: Vec<EntityHeader>,
    pub skipped: Vec<EntityHeader>,
    pub rejected: Vec<String>,  // e.g. ambiguous or inconsistent
    pub discarded: Vec<String>, // e.g. nonexistent and need to be created
}

pub fn create_track(
    conn: &SqlitePooledConnection,
    new_track: Track,
    body_data: EntityBodyData,
) -> RepoResult<EntityHeader> {
    let repo = Repository::new(&*conn);
    let hdr = EntityHeader::initial_random();
    let entity = Entity::new(hdr.clone(), new_track);
    conn.transaction::<_, Error, _>(|| repo.insert_track(entity, body_data).map(|()| hdr))
}

pub fn update_track(
    conn: &SqlitePooledConnection,
    track: Entity,
    body_data: EntityBodyData,
) -> RepoResult<EntityRevisionUpdateResult> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| repo.update_track(track, body_data))
}

pub fn delete_track(conn: &SqlitePooledConnection, uid: &EntityUid) -> RepoResult<Option<()>> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| repo.delete_track(uid))
}

pub fn load_track(
    conn: &SqlitePooledConnection,
    uid: &EntityUid,
) -> RepoResult<Option<EntityData>> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| repo.load_track(uid))
}

pub fn load_tracks(
    conn: &SqlitePooledConnection,
    uids: impl Iterator<Item = EntityUid>,
) -> RepoResult<Vec<EntityData>> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| repo.load_tracks(&uids.collect::<Vec<_>>()))
}

pub fn list_tracks(
    conn: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
) -> RepoResult<Vec<EntityData>> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| {
        repo.search_tracks(collection_uid.as_ref(), pagination, Default::default())
    })
}

pub fn search_tracks(
    conn: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
    params: SearchParams,
) -> RepoResult<Vec<EntityData>> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| {
        repo.search_tracks(collection_uid.as_ref(), pagination, params)
    })
}

pub fn locate_tracks(
    conn: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
    params: LocateParams,
) -> RepoResult<Vec<EntityData>> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| {
        repo.locate_tracks(collection_uid.as_ref(), pagination, params)
    })
}

pub fn replace_tracks(
    conn: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    mode: ReplaceMode,
    replacements: impl Iterator<Item = TrackReplacement>,
) -> RepoResult<ReplacedTracks> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| {
        let mut results = ReplacedTracks::default();
        for replacement in replacements {
            let body_data = json::serialize_entity_body_data(&replacement.track)?;
            let (data_fmt, data_ver, _) = body_data;
            let media_uri = replacement.media_uri;
            let replace_result = repo.replace_track(
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
    })
}

pub fn purge_tracks(
    conn: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    uri_predicates: impl IntoIterator<Item = UriPredicate>,
) -> RepoResult<()> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| {
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
            let entities =
                repo.locate_tracks(collection_uid.as_ref(), Default::default(), locate_params)?;
            log::debug!(
                "Found {} track(s) that match {:?} as candidates for purging",
                entities.len(),
                uri_predicate,
            );
            for entity in entities.into_iter() {
                let Entity { hdr, mut body, .. } = json::deserialize_entity_from_data(entity)?;
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
                        repo.delete_track(&hdr.uid)?;
                    } else {
                        log::debug!(
                            "Updating track {} after purging {} of {} media source(s)",
                            hdr.uid,
                            purged,
                            purged + body.media_sources.len(),
                        );
                        // TODO: Avoid temporary clone
                        let json_data = json::serialize_entity_body_data(&body.clone().into())?;
                        let entity = Entity::new(hdr, body);
                        let _update_result = repo.update_track(entity, json_data)?;
                        debug_assert!(_update_result.is_updated());
                    }
                } else {
                    log::debug!("No media sources purged from track {}", hdr.uid);
                }
            }
        }
        Ok(())
    })
}

pub fn relocate_tracks(
    conn: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    uri_relocations: impl IntoIterator<Item = UriRelocation>,
) -> RepoResult<()> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| {
        for uri_relocation in uri_relocations {
            let locate_params = match &uri_relocation.predicate {
                UriPredicate::Prefix(uri_prefix) => LocateParams {
                    media_uri: StringPredicate::StartsWith(uri_prefix.to_owned()),
                },
                UriPredicate::Exact(uri) => LocateParams {
                    media_uri: StringPredicate::Equals(uri.to_owned()),
                },
            };
            let tracks =
                repo.locate_tracks(collection_uid.as_ref(), Default::default(), locate_params)?;
            log::debug!(
                "Found {} track(s) that match {:?} as candidates for relocating",
                tracks.len(),
                uri_relocation.predicate,
            );
            for entity_data in tracks {
                let Entity {
                    hdr,
                    body: mut track,
                    ..
                } = json::deserialize_entity_from_data(entity_data)?;
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
                    let json_data = json::serialize_entity_body_data(&track.clone().into())?;
                    let entity = Entity::new(hdr, track);
                    let _update_result = repo.update_track(entity, json_data)?;
                    debug_assert!(_update_result.is_updated());
                } else {
                    log::debug!("No sources relocated for track {}", hdr.uid);
                }
            }
        }
        Ok(())
    })
}

pub fn count_tracks_by_album(
    conn: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
    params: &CountTracksByAlbumParams,
) -> RepoResult<Vec<AlbumCountResults>> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| {
        repo.count_tracks_by_album(collection_uid.as_ref(), params, pagination)
    })
}

pub fn count_tracks_by_tag(
    conn: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
    mut params: TagCountParams,
) -> RepoResult<Vec<TagAvgScoreCount>> {
    params.dedup_facets();
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| {
        repo.count_tracks_by_tag(collection_uid.as_ref(), &params, pagination)
    })
}

pub fn count_tracks_by_tag_facet(
    conn: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    pagination: Pagination,
    mut params: TagFacetCountParams,
) -> RepoResult<Vec<TagFacetCount>> {
    params.dedup_facets();
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| {
        repo.count_tracks_by_tag_facet(collection_uid.as_ref(), &params, pagination)
    })
}
