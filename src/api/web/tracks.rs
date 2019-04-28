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
        CountTagFacetsParams, CountTagsParams, CountTrackAlbumsParams, LocateTracksParams,
        Pagination, PaginationLimit, PaginationOffset, ReplaceTracksParams, ReplacedTracks,
        SearchTracksParams, TagCount, TagFacetCount,
    },
    storage::track::TrackRepository,
};

use futures::future::{self, Future};

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
            .map(|val| {
                warp::reply::with_status(warp::reply::json(&val), warp::http::StatusCode::CREATED)
            })
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
                    let next_header = aoide_core::entity::EntityHeader::new(uid, next_revision);
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
                    res.map(|()| warp::http::StatusCode::NO_CONTENT)
                        .unwrap_or(warp::http::StatusCode::NOT_FOUND),
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

    pub fn handle_albums_count(
        &self,
        query_params: TracksQueryParams,
        count_params: CountTrackAlbumsParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        count_albums(&self.db, query_params, &count_params)
            .map(|val| warp::reply::json(&val))
            .map_err(warp::reject::custom)
    }

    pub fn handle_tags_count(
        &self,
        query_params: TracksQueryParams,
        count_params: CountTagsParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        count_tags(&self.db, query_params, count_params)
            .map(|val| warp::reply::json(&val))
            .map_err(warp::reject::custom)
    }

    pub fn handle_tags_facets_count(
        &self,
        query_params: TracksQueryParams,
        count_params: CountTagFacetsParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        count_tag_facets(&self.db, query_params, count_params)
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

fn count_albums(
    pooled_connection: &SqlitePooledConnection,
    query: TracksQueryParams,
    params: &CountTrackAlbumsParams,
) -> impl Future<Item = Vec<TrackAlbumCount>, Error = Error> {
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_albums(query.collection_uid.as_ref(), params, query.pagination())
    }))
}

fn count_tags(
    pooled_connection: &SqlitePooledConnection,
    query: TracksQueryParams,
    params: CountTagsParams,
) -> impl Future<Item = Vec<TagCount>, Error = Error> {
    let include_non_faceted_tags = params.include_non_faceted_tags;
    let facets = params.facets.map(|mut facets| {
        facets.sort();
        facets.dedup();
        facets
    });
    let facets = facets.as_ref().map(|facets| {
        facets
            .iter()
            .map(AsRef::as_ref)
            .map(String::as_str)
            .collect()
    });
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_tags(
            query.collection_uid.as_ref(),
            facets.as_ref().map(Vec::as_slice),
            include_non_faceted_tags,
            query.pagination(),
        )
    }))
}

fn count_tag_facets(
    pooled_connection: &SqlitePooledConnection,
    query: TracksQueryParams,
    params: CountTagFacetsParams,
) -> impl Future<Item = Vec<TagFacetCount>, Error = Error> {
    let facets = params.facets.map(|mut facets| {
        facets.sort();
        facets.dedup();
        facets
    });
    let facets = facets.as_ref().map(|facets| {
        facets
            .iter()
            .map(AsRef::as_ref)
            .map(String::as_str)
            .collect()
    });
    let repository = TrackRepository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_tag_facets(
            query.collection_uid.as_ref(),
            facets.as_ref().map(Vec::as_slice),
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
