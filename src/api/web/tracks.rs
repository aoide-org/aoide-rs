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

use aoide_storage::{
    api::{
        serde::{SerializationFormat, SerializedEntity},
        track::{TrackAlbums, TrackTags, Tracks, TracksResult},
        CountTracksByAlbumParams, CountTracksByTagFacetParams, CountTracksByTagParams,
        LocateTracksParams, Pagination, PaginationLimit, PaginationOffset, ReplaceTracksParams,
        ReplacedTracks, SearchTracksParams, StringPredicate, TagCount, TagFacetCount, UriPredicate,
        UriRelocation,
    },
    storage::track::TrackRepository,
};

use futures::future::{self, Future};
use warp::http::StatusCode;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracksQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_uid: Option<EntityUid>,

    // Flatting of Pagination does not work as expected:
    // https://github.com/serde-rs/serde/issues/1183
    // Workaround: Inline all parameters manually
    //#[serde(flatten)]
    //pub pagination: Pagination,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PaginationOffset>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PaginationLimit>,
}

impl TracksQueryParams {
    pub fn pagination(&self) -> Pagination {
        Pagination {
            offset: self.offset,
            limit: self.limit,
        }
    }
}

fn reply_with_json_content(reply: impl warp::Reply) -> impl warp::Reply {
    warp::reply::with_header(reply, "Content-Type", "application/json")
}

pub struct TracksHandler {
    db: SqlitePooledConnection,
}

impl TracksHandler {
    pub fn new(db: SqlitePooledConnection) -> Self {
        Self { db }
    }

    pub fn handle_create(
        &self,
        new_track: Track,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        create_track(&self.db, new_track, SerializationFormat::JSON)
            .map_err(warp::reject::custom)
            .map(|val| warp::reply::with_status(warp::reply::json(&val), StatusCode::CREATED))
    }

    pub fn handle_update(
        &self,
        uid: EntityUid,
        track: TrackEntity,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        if uid != *track.header().uid() {
            return Err(warp::reject::custom(failure::format_err!(
                "Mismatching UIDs: {} <> {}",
                uid,
                track.header().uid(),
            )));
        }
        update_track(&self.db, track, SerializationFormat::JSON)
            .and_then(move |res| match res {
                (_, Some(next_revision)) => {
                    let next_header = aoide_domain::entity::EntityHeader::new(uid, next_revision);
                    Ok(warp::reply::json(&next_header))
                }
                (_, None) => Err(failure::format_err!(
                    "Inexistent entity or revision conflict"
                )),
            })
            .map_err(warp::reject::custom)
    }

    pub fn handle_delete(
        &self,
        uid: EntityUid,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        delete_track(&self.db, &uid)
            .map_err(warp::reject::custom)
            .map(|res| {
                warp::reply::with_status(
                    warp::reply(),
                    res.map(|()| StatusCode::NO_CONTENT)
                        .unwrap_or(StatusCode::NOT_FOUND),
                )
            })
    }

    pub fn handle_load(&self, uid: EntityUid) -> Result<impl warp::Reply, warp::reject::Rejection> {
        load_track(&self.db, &uid)
            .map_err(warp::reject::custom)
            .and_then(|res| match res {
                Some(val) => {
                    let mime_type: mime::Mime = val.format.into();
                    let body: Vec<u8> = val.blob;
                    Ok(warp::reply::with_header(
                        body,
                        "Content-Type",
                        mime_type.to_string().as_str(),
                    ))
                }
                None => Err(warp::reject::not_found()),
            })
    }

    pub fn handle_list(
        &self,
        query_params: TracksQueryParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        list_tracks(&self.db, &query_params)
            .and_then(|reply| {
                aoide_storage::api::serde::SerializedEntity::slice_to_json_array(&reply)
            })
            .map(reply_with_json_content)
            .map_err(warp::reject::custom)
    }

    pub fn handle_search(
        &self,
        query_params: TracksQueryParams,
        search_params: SearchTracksParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        search_tracks(&self.db, query_params, search_params)
            .and_then(|reply| {
                aoide_storage::api::serde::SerializedEntity::slice_to_json_array(&reply)
            })
            .map(reply_with_json_content)
            .map_err(warp::reject::custom)
    }

    pub fn handle_locate(
        &self,
        query_params: TracksQueryParams,
        locate_params: LocateTracksParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        locate_tracks(&self.db, query_params, locate_params)
            .and_then(|reply| {
                aoide_storage::api::serde::SerializedEntity::slice_to_json_array(&reply)
            })
            .map(reply_with_json_content)
            .map_err(warp::reject::custom)
    }

    pub fn handle_replace(
        &self,
        query_params: TracksQueryParams,
        replace_params: ReplaceTracksParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        replace_tracks(
            &self.db,
            query_params,
            replace_params,
            SerializationFormat::JSON,
        )
        .map(|val| warp::reply::json(&val))
        .map_err(warp::reject::custom)
    }

    pub fn handle_purge(
        &self,
        query_params: TracksQueryParams,
        uri_predicates: impl IntoIterator<Item = UriPredicate>,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        purge_tracks(&self.db, query_params.collection_uid, uri_predicates)
            .map(|()| StatusCode::NO_CONTENT)
            .map_err(warp::reject::custom)
    }

    pub fn handle_relocate(
        &self,
        query_params: TracksQueryParams,
        uri_relocations: impl IntoIterator<Item = UriRelocation>,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        relocate_tracks(&self.db, query_params.collection_uid, uri_relocations)
            .map(|()| StatusCode::NO_CONTENT)
            .map_err(warp::reject::custom)
    }

    pub fn handle_albums_count_tracks(
        &self,
        query_params: TracksQueryParams,
        count_params: CountTracksByAlbumParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        count_tracks_by_album(&self.db, query_params, &count_params)
            .map(|val| warp::reply::json(&val))
            .map_err(warp::reject::custom)
    }

    pub fn handle_tags_count_tracks(
        &self,
        query_params: TracksQueryParams,
        count_params: CountTracksByTagParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        count_tracks_by_tag(&self.db, query_params, count_params)
            .map(|val| warp::reply::json(&val))
            .map_err(warp::reject::custom)
    }

    pub fn handle_tags_facets_count_tracks(
        &self,
        query_params: TracksQueryParams,
        count_params: CountTracksByTagFacetParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        count_tracks_by_tag_facet(&self.db, query_params, count_params)
            .map(|val| warp::reply::json(&val))
            .map_err(warp::reject::custom)
    }
}

fn create_track(
    db: &SqlitePooledConnection,
    new_track: Track,
    format: SerializationFormat,
) -> TracksResult<TrackEntity> {
    let repository = TrackRepository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.create_entity(new_track, format))
}

fn update_track(
    db: &SqlitePooledConnection,
    track: TrackEntity,
    format: SerializationFormat,
) -> TracksResult<(EntityRevision, Option<EntityRevision>)> {
    let repository = TrackRepository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.update_entity(track, format))
}

fn delete_track(db: &SqlitePooledConnection, uid: &EntityUid) -> TracksResult<Option<()>> {
    let repository = TrackRepository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.delete_entity(uid))
}

fn load_track(
    pooled_connection: &SqlitePooledConnection,
    uid: &EntityUid,
) -> TracksResult<Option<SerializedEntity>> {
    let repository = TrackRepository::new(&*pooled_connection);
    pooled_connection.transaction::<_, Error, _>(|| repository.load_entity(uid))
}

fn list_tracks(
    pooled_connection: &SqlitePooledConnection,
    query: &TracksQueryParams,
) -> impl Future<Item = Vec<SerializedEntity>, Error = Error> {
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.search_entities(
            query.collection_uid.as_ref(),
            query.pagination(),
            Default::default(),
        )
    }))
}

fn search_tracks(
    pooled_connection: &SqlitePooledConnection,
    query: TracksQueryParams,
    params: SearchTracksParams,
) -> impl Future<Item = Vec<SerializedEntity>, Error = Error> {
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.search_entities(query.collection_uid.as_ref(), query.pagination(), params)
    }))
}

fn locate_tracks(
    pooled_connection: &SqlitePooledConnection,
    query: TracksQueryParams,
    params: LocateTracksParams,
) -> impl Future<Item = Vec<SerializedEntity>, Error = Error> {
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.locate_entities(query.collection_uid.as_ref(), query.pagination(), params)
    }))
}

fn replace_tracks(
    pooled_connection: &SqlitePooledConnection,
    query: TracksQueryParams,
    params: ReplaceTracksParams,
    format: SerializationFormat,
) -> impl Future<Item = ReplacedTracks, Error = Error> {
    debug_assert!(query.offset.is_none()); // unused
    debug_assert!(query.limit.is_none()); // unused
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.replace_entities(query.collection_uid.as_ref(), params, format)
    }))
}

fn purge_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    uri_predicates: impl IntoIterator<Item = UriPredicate>,
) -> impl Future<Item = (), Error = Error> {
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        for uri_predicate in uri_predicates {
            let locate_params = match &uri_predicate {
                UriPredicate::Prefix(uri_prefix) => LocateTracksParams {
                    uri: StringPredicate::StartsWith(uri_prefix.to_owned()),
                },
                UriPredicate::Exact(uri) => LocateTracksParams {
                    uri: StringPredicate::Equals(uri.to_owned()),
                },
            };
            let tracks = repository.locate_entities(
                collection_uid.as_ref(),
                Default::default(),
                locate_params,
            )?;
            log::debug!(
                "Found {} track(s) that match {:?} as candidates for purging",
                tracks.len(),
                uri_predicate,
            );
            for track in tracks {
                let format = track.format;
                let mut track: TrackEntity = track.deserialize()?;
                let purged = match &uri_predicate {
                    UriPredicate::Prefix(ref uri_prefix) => {
                        track.body_mut().purge_source_by_uri_prefix(uri_prefix)
                    }
                    UriPredicate::Exact(ref uri) => track.body_mut().purge_source_by_uri(uri),
                };
                if purged > 0 {
                    if track.body().sources.is_empty() {
                        log::debug!(
                            "Deleting track {} after purging all (= {}) sources",
                            track.header().uid(),
                            purged,
                        );
                        repository.delete_entity(track.header().uid())?;
                    } else {
                        log::debug!(
                            "Updating track {} after purging {} of {} source(s)",
                            track.header().uid(),
                            purged,
                            purged + track.body().sources.len(),
                        );
                        let updated = repository.update_entity(track, format)?;
                        debug_assert!(updated.1.is_some());
                    }
                } else {
                    log::debug!("No sources purged from track {}", track.header().uid());
                }
            }
        }
        Ok(())
    }))
}

fn relocate_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<EntityUid>,
    uri_relocations: impl IntoIterator<Item = UriRelocation>,
) -> impl Future<Item = (), Error = Error> {
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        for uri_relocation in uri_relocations {
            let locate_params = match &uri_relocation.predicate {
                UriPredicate::Prefix(uri_prefix) => LocateTracksParams {
                    uri: StringPredicate::StartsWith(uri_prefix.to_owned()),
                },
                UriPredicate::Exact(uri) => LocateTracksParams {
                    uri: StringPredicate::Equals(uri.to_owned()),
                },
            };
            let tracks = repository.locate_entities(
                collection_uid.as_ref(),
                Default::default(),
                locate_params,
            )?;
            log::debug!(
                "Found {} track(s) that match {:?} as candidates for relocating",
                tracks.len(),
                uri_relocation.predicate,
            );
            for track in tracks {
                let format = track.format;
                let mut track: TrackEntity = track.deserialize()?;
                let relocated = match &uri_relocation.predicate {
                    UriPredicate::Prefix(uri_prefix) => track
                        .body_mut()
                        .relocate_source_by_uri_prefix(uri_prefix, &uri_relocation.replacement),
                    UriPredicate::Exact(uri) => track
                        .body_mut()
                        .relocate_source_by_uri(uri, &uri_relocation.replacement),
                };
                if relocated > 0 {
                    log::debug!(
                        "Updating track {} after relocating {} source(s)",
                        track.header().uid(),
                        relocated,
                    );
                    let updated = repository.update_entity(track, format)?;
                    debug_assert!(updated.1.is_some());
                } else {
                    log::debug!("No sources relocated for track {}", track.header().uid());
                }
            }
        }
        Ok(())
    }))
}

fn count_tracks_by_album(
    pooled_connection: &SqlitePooledConnection,
    query: TracksQueryParams,
    params: &CountTracksByAlbumParams,
) -> impl Future<Item = Vec<AlbumTracksCount>, Error = Error> {
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_tracks_by_album(query.collection_uid.as_ref(), params, query.pagination())
    }))
}

fn count_tracks_by_tag(
    pooled_connection: &SqlitePooledConnection,
    query: TracksQueryParams,
    mut params: CountTracksByTagParams,
) -> impl Future<Item = Vec<TagCount>, Error = Error> {
    params.dedup_facets();
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_tracks_by_tag(query.collection_uid.as_ref(), &params, query.pagination())
    }))
}

fn count_tracks_by_tag_facet(
    pooled_connection: &SqlitePooledConnection,
    query: TracksQueryParams,
    mut params: CountTracksByTagFacetParams,
) -> impl Future<Item = Vec<TagFacetCount>, Error = Error> {
    params.dedup_facets();
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_tracks_by_tag_facet(
            query.collection_uid.as_ref(),
            &params,
            query.pagination(),
        )
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencode_tracks_query_params() {
        let collection_uid =
            EntityUid::decode_from_str("DNGwV8sS9XS2GAxfEvgW2NMFxDHwi81CC").unwrap();

        let query = TracksQueryParams {
            collection_uid: Some(collection_uid),
            offset: Some(0),
            limit: Some(2),
        };
        let query_urlencoded = "collectionUid=DNGwV8sS9XS2GAxfEvgW2NMFxDHwi81CC&offset=0&limit=2";
        assert_eq!(
            query_urlencoded,
            serde_urlencoded::to_string(&query).unwrap()
        );
        assert_eq!(
            query,
            serde_urlencoded::from_str::<TracksQueryParams>(query_urlencoded).unwrap()
        );

        let query = TracksQueryParams {
            collection_uid: Some(collection_uid),
            offset: None,
            limit: Some(2),
        };
        let query_urlencoded = "collectionUid=DNGwV8sS9XS2GAxfEvgW2NMFxDHwi81CC&limit=2";
        assert_eq!(
            query_urlencoded,
            serde_urlencoded::to_string(&query).unwrap()
        );
        assert_eq!(
            query,
            serde_urlencoded::from_str::<TracksQueryParams>(query_urlencoded).unwrap()
        );

        let query = TracksQueryParams {
            collection_uid: Some(collection_uid),
            offset: None,
            limit: None,
        };
        let query_urlencoded = "collectionUid=DNGwV8sS9XS2GAxfEvgW2NMFxDHwi81CC";
        assert_eq!(
            query_urlencoded,
            serde_urlencoded::to_string(&query).unwrap()
        );
        assert_eq!(
            query,
            serde_urlencoded::from_str::<TracksQueryParams>(query_urlencoded).unwrap()
        );

        let query = TracksQueryParams {
            collection_uid: None,
            offset: Some(1),
            limit: None,
        };
        let query_urlencoded = "offset=1";
        assert_eq!(
            query_urlencoded,
            serde_urlencoded::to_string(&query).unwrap()
        );
        assert_eq!(
            query,
            serde_urlencoded::from_str::<TracksQueryParams>(query_urlencoded).unwrap()
        );
    }
}
