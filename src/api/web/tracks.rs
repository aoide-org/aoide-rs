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

use crate::usecases::tracks::{
    ReplacedTracks as UcReplacedTracks, TrackReplacement as UcTrackReplacement, *,
};

mod json {
    pub use super::super::json::*;
    pub use crate::usecases::tracks::json::*;
}

mod _serde {
    pub use aoide_core_serde::{
        entity::{EntityHeader, EntityUid},
        track::Entity,
    };
}

// NOTE: This additional module is just a workaround, because
// otherwise _serde::EntityUid (see above) is not found!?!?
mod _serde2 {
    pub use aoide_core_serde::entity::EntityUid;
}

mod _repo {
    pub use aoide_repo::{
        tag::{
            AvgScoreCount as TagAvgScoreCount, CountParams as TagCountParams,
            FacetCount as TagFacetCount, FacetCountParams as TagFacetCountParams,
            Filter as TagFilter, SortField as TagSortField, SortOrder as TagSortOrder,
        },
        track::{
            AlbumCountResults, CountTracksByAlbumParams, LocateParams, NumericField,
            NumericFieldFilter, PhraseFieldFilter, ReplaceMode, ReplaceResult, SearchFilter,
            SearchParams, SortField, SortOrder, StringField,
        },
        util::{UriPredicate, UriRelocation},
        FilterModifier, NumericPredicate, NumericValue, SortDirection, StringFilter,
        StringPredicate,
    };
}

use aoide_core::{
    entity::{EntityHeader, EntityRevisionUpdateResult, EntityUid},
    tag::ScoreValue as TagScoreValue,
    track::{
        release::{ReleaseDate, YYYYMMDD},
        Entity,
    },
};

use aoide_repo::{Pagination, PaginationLimit, PaginationOffset};

use aoide_core_serde::track::Track;

use futures::future::Future;
use warp::http::StatusCode;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TracksQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_uid: Option<_serde2::EntityUid>,

    // Flattening of Pagination does not work as expected:
    // https://github.com/serde-rs/serde/issues/1183
    // Workaround: Inline all parameters manually
    //#[serde(flatten)]
    //pub pagination: Pagination,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PaginationOffset>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PaginationLimit>,
}

impl From<TracksQueryParams> for (Option<EntityUid>, Pagination) {
    fn from(from: TracksQueryParams) -> Self {
        let collection_uid = from.collection_uid.map(Into::into);
        let pagination = Pagination {
            offset: from.offset,
            limit: from.limit,
        };
        (collection_uid, pagination)
    }
}

/// Predicates for matching strings
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StringPredicate {
    StartsWith(String),
    StartsNotWith(String),
    EndsWith(String),
    EndsNotWith(String),
    Contains(String),
    ContainsNot(String),
    Matches(String),
    MatchesNot(String),
    Equals(String),
    EqualsNot(String),
}

impl From<StringPredicate> for _repo::StringPredicate {
    fn from(from: StringPredicate) -> Self {
        use _repo::StringPredicate::*;
        match from {
            StringPredicate::StartsWith(s) => StartsWith(s),
            StringPredicate::StartsNotWith(s) => StartsNotWith(s),
            StringPredicate::EndsWith(s) => EndsWith(s),
            StringPredicate::EndsNotWith(s) => EndsNotWith(s),
            StringPredicate::Contains(s) => Contains(s),
            StringPredicate::ContainsNot(s) => ContainsNot(s),
            StringPredicate::Matches(s) => Matches(s),
            StringPredicate::MatchesNot(s) => MatchesNot(s),
            StringPredicate::Equals(s) => Equals(s),
            StringPredicate::EqualsNot(s) => EqualsNot(s),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LocateParams {
    pub media_uri: StringPredicate,
}

impl From<LocateParams> for _repo::LocateParams {
    fn from(from: LocateParams) -> Self {
        Self {
            media_uri: from.media_uri.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortField {
    InCollectionSince,
    LastRevisionedAt,
    TrackTitle,
    TrackArtist,
    TrackNumber,
    TrackTotal,
    DiscNumber,
    DiscTotal,
    AlbumTitle,
    AlbumArtist,
    ReleaseDate,
    MusicTempo,
    MusicKey,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,

    #[serde(rename = "dsc")]
    Descending,
}

impl From<SortDirection> for _repo::SortDirection {
    fn from(from: SortDirection) -> Self {
        use _repo::SortDirection::*;
        match from {
            SortDirection::Ascending => Ascending,
            SortDirection::Descending => Descending,
        }
    }
}

impl From<SortField> for _repo::SortField {
    fn from(from: SortField) -> Self {
        use _repo::SortField::*;
        match from {
            SortField::InCollectionSince => InCollectionSince,
            SortField::LastRevisionedAt => LastRevisionedAt,
            SortField::TrackTitle => TrackTitle,
            SortField::TrackArtist => TrackArtist,
            SortField::TrackNumber => TrackNumber,
            SortField::TrackTotal => TrackTotal,
            SortField::DiscNumber => DiscNumber,
            SortField::DiscTotal => DiscTotal,
            SortField::AlbumTitle => AlbumTitle,
            SortField::AlbumArtist => AlbumArtist,
            SortField::ReleaseDate => ReleaseDate,
            SortField::MusicTempo => MusicTempo,
            SortField::MusicKey => MusicKey,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct SortOrder(SortField, SortDirection);

impl From<SortOrder> for _repo::SortOrder {
    fn from(from: SortOrder) -> Self {
        Self {
            field: from.0.into(),
            direction: from.1.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FilterModifier {
    Complement,
}

impl From<FilterModifier> for _repo::FilterModifier {
    fn from(from: FilterModifier) -> Self {
        use _repo::FilterModifier::*;
        match from {
            FilterModifier::Complement => Complement,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringFilter {
    #[serde(skip_serializing_if = "Option::None")]
    pub modifier: Option<FilterModifier>,

    #[serde(skip_serializing_if = "Option::None")]
    pub value: Option<StringPredicate>,
}

impl From<StringFilter> for _repo::StringFilter {
    fn from(from: StringFilter) -> Self {
        Self {
            modifier: from.modifier.map(Into::into),
            value: from.value.map(Into::into),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StringField {
    MediaUri,
    MediaType,
    TrackTitle,
    TrackArtist,
    TrackComposer,
    AlbumTitle,
    AlbumArtist,
}

impl From<StringField> for _repo::StringField {
    fn from(from: StringField) -> Self {
        use _repo::StringField::*;
        match from {
            StringField::MediaUri => MediaUri,
            StringField::MediaType => MediaType,
            StringField::TrackTitle => TrackTitle,
            StringField::TrackArtist => TrackArtist,
            StringField::TrackComposer => TrackComposer,
            StringField::AlbumTitle => AlbumTitle,
            StringField::AlbumArtist => AlbumArtist,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NumericField {
    AudioBitRate,
    AudioChannelCount,
    AudioDuration,
    AudioSampleRate,
    AudioLoudness,
    TrackNumber,
    TrackTotal,
    DiscNumber,
    DiscTotal,
    ReleaseDate,
    MusicTempo,
    MusicKey,
}

impl From<NumericField> for _repo::NumericField {
    fn from(from: NumericField) -> Self {
        use _repo::NumericField::*;
        match from {
            NumericField::AudioBitRate => AudioBitRate,
            NumericField::AudioChannelCount => AudioChannelCount,
            NumericField::AudioDuration => AudioDuration,
            NumericField::AudioSampleRate => AudioSampleRate,
            NumericField::AudioLoudness => AudioLoudness,
            NumericField::TrackNumber => TrackNumber,
            NumericField::TrackTotal => TrackTotal,
            NumericField::DiscNumber => DiscNumber,
            NumericField::DiscTotal => DiscTotal,
            NumericField::ReleaseDate => ReleaseDate,
            NumericField::MusicTempo => MusicTempo,
            NumericField::MusicKey => MusicKey,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum NumericPredicate {
    #[serde(rename = "lt")]
    LessThan(_repo::NumericValue),

    #[serde(rename = "le")]
    LessOrEqual(_repo::NumericValue),

    #[serde(rename = "gt")]
    GreaterThan(_repo::NumericValue),

    #[serde(rename = "ge")]
    GreaterOrEqual(_repo::NumericValue),

    #[serde(rename = "eq")]
    Equal(Option<_repo::NumericValue>),

    #[serde(rename = "ne")]
    NotEqual(Option<_repo::NumericValue>),
}

impl From<NumericPredicate> for _repo::NumericPredicate {
    fn from(from: NumericPredicate) -> Self {
        use _repo::NumericPredicate::*;
        match from {
            NumericPredicate::LessThan(val) => LessThan(val),
            NumericPredicate::LessOrEqual(val) => LessOrEqual(val),
            NumericPredicate::GreaterThan(val) => GreaterThan(val),
            NumericPredicate::GreaterOrEqual(val) => GreaterOrEqual(val),
            NumericPredicate::Equal(val) => Equal(val),
            NumericPredicate::NotEqual(val) => NotEqual(val),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct NumericFieldFilter(NumericField, NumericPredicate);

impl From<NumericFieldFilter> for _repo::NumericFieldFilter {
    fn from(from: NumericFieldFilter) -> Self {
        Self {
            field: from.0.into(),
            value: from.1.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PhraseFieldFilter(Vec<StringField>, Vec<String>);

impl From<PhraseFieldFilter> for _repo::PhraseFieldFilter {
    fn from(from: PhraseFieldFilter) -> Self {
        Self {
            fields: from.0.into_iter().map(Into::into).collect(),
            terms: from.1,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagFilter {
    #[serde(skip_serializing_if = "Option::None")]
    pub modifier: Option<FilterModifier>,

    // Facets are always matched with equals. Use an empty vector
    // for matching only tags without a facet.
    #[serde(skip_serializing_if = "Option::None")]
    pub facets: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::None")]
    pub label: Option<StringPredicate>,

    #[serde(skip_serializing_if = "Option::None")]
    pub score: Option<NumericPredicate>,
}

impl From<TagFilter> for _repo::TagFilter {
    fn from(from: TagFilter) -> Self {
        Self {
            modifier: from.modifier.map(Into::into),
            facets: from.facets,
            label: from.label.map(Into::into),
            score: from.score.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchFilter {
    Phrase(PhraseFieldFilter),
    Numeric(NumericFieldFilter),
    Tag(TagFilter),
    MarkerLabel(StringFilter),
    All(Vec<SearchFilter>),
    Any(Vec<SearchFilter>),
    Not(Box<SearchFilter>),
}

impl From<SearchFilter> for _repo::SearchFilter {
    fn from(from: SearchFilter) -> Self {
        use _repo::SearchFilter::*;
        match from {
            SearchFilter::Phrase(from) => Phrase(from.into()),
            SearchFilter::Numeric(from) => Numeric(from.into()),
            SearchFilter::Tag(from) => Tag(from.into()),
            SearchFilter::MarkerLabel(from) => MarkerLabel(from.into()),
            SearchFilter::All(from) => All(from.into_iter().map(Into::into).collect()),
            SearchFilter::Any(from) => Any(from.into_iter().map(Into::into).collect()),
            SearchFilter::Not(from) => Not(Box::new((*from).into())),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SearchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<SearchFilter>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<SortOrder>,
}

impl From<SearchParams> for _repo::SearchParams {
    fn from(from: SearchParams) -> Self {
        Self {
            filter: from.filter.map(Into::into),
            ordering: from.ordering.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TagSortField {
    Facet,
    Label,
    Score,
    Count,
}

impl From<TagSortField> for _repo::TagSortField {
    fn from(from: TagSortField) -> Self {
        use _repo::TagSortField::*;
        match from {
            TagSortField::Facet => Facet,
            TagSortField::Label => Label,
            TagSortField::Score => Score,
            TagSortField::Count => Count,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct TagSortOrder(TagSortField, SortDirection);

impl From<TagSortOrder> for _repo::TagSortOrder {
    fn from(from: TagSortOrder) -> Self {
        Self {
            field: from.0.into(),
            direction: from.1.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagCountParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_non_faceted_tags: Option<bool>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<TagSortOrder>,
}

impl From<TagCountParams> for _repo::TagCountParams {
    fn from(from: TagCountParams) -> Self {
        Self {
            facets: from.facets.map(|facets| {
                facets
                    .into_iter()
                    .filter_map(|facet| facet.parse().ok())
                    .collect()
            }),
            include_non_faceted_tags: from.include_non_faceted_tags,
            ordering: from.ordering.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TagAvgScoreCount {
    #[serde(rename = "f", skip_serializing_if = "Option::is_none")]
    pub facet: Option<String>,

    #[serde(rename = "l", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(rename = "s")]
    pub avg_score: TagScoreValue,

    #[serde(rename = "n")]
    pub total_count: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagFacetCountParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<TagSortOrder>,
}

impl From<TagFacetCountParams> for _repo::TagFacetCountParams {
    fn from(from: TagFacetCountParams) -> Self {
        Self {
            facets: from.facets.map(|facets| {
                facets
                    .into_iter()
                    .filter_map(|facet| facet.parse().ok())
                    .collect()
            }),
            ordering: from.ordering.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TagFacetCount(String, usize);

impl From<_repo::TagFacetCount> for TagFacetCount {
    fn from(from: _repo::TagFacetCount) -> Self {
        Self(from.facet.into(), from.total_count)
    }
}

impl From<_repo::TagAvgScoreCount> for TagAvgScoreCount {
    fn from(from: _repo::TagAvgScoreCount) -> Self {
        Self {
            facet: from.facet.map(Into::into),
            label: from.label.map(Into::into),
            avg_score: from.avg_score.into(),
            total_count: from.total_count,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CountTracksByAlbumParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_release_date: Option<YYYYMMDD>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_release_date: Option<YYYYMMDD>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<SortOrder>,
}

impl From<CountTracksByAlbumParams> for _repo::CountTracksByAlbumParams {
    fn from(from: CountTracksByAlbumParams) -> Self {
        Self {
            min_release_date: from.min_release_date.map(ReleaseDate::new),
            max_release_date: from.max_release_date.map(ReleaseDate::new),
            ordering: from.ordering.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AlbumTrackCount {
    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "a", skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,

    #[serde(rename = "d", skip_serializing_if = "Option::is_none")]
    pub release_date: Option<YYYYMMDD>,

    #[serde(rename = "n")]
    pub total_count: usize,
}

impl From<_repo::AlbumCountResults> for AlbumTrackCount {
    fn from(from: _repo::AlbumCountResults) -> Self {
        Self {
            title: from.title,
            artist: from.artist,
            release_date: from.release_date.map(Into::into),
            total_count: from.total_count,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
enum ReplaceMode {
    UpdateOnly,
    UpdateOrCreate,
}

impl From<ReplaceMode> for _repo::ReplaceMode {
    fn from(from: ReplaceMode) -> Self {
        use _repo::ReplaceMode::*;
        match from {
            ReplaceMode::UpdateOnly => UpdateOnly,
            ReplaceMode::UpdateOrCreate => UpdateOrCreate,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReplaceTracksParams {
    mode: ReplaceMode,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    replacements: Vec<TrackReplacement>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackReplacement {
    // The URI for locating any existing track that is supposed
    // to replaced by the provided track.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    media_uri: String,

    track: Track,
}

impl From<TrackReplacement> for UcTrackReplacement {
    fn from(from: TrackReplacement) -> Self {
        Self {
            media_uri: from.media_uri,
            track: from.track,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReplacedTracks {
    pub created: Vec<_serde::EntityHeader>,
    pub updated: Vec<_serde::EntityHeader>,
    pub skipped: Vec<_serde::EntityHeader>,
    pub rejected: Vec<String>,  // e.g. ambiguous or inconsistent
    pub discarded: Vec<String>, // e.g. nonexistent and need to be created
}

impl From<UcReplacedTracks> for ReplacedTracks {
    fn from(from: UcReplacedTracks) -> Self {
        Self {
            created: from.created.into_iter().map(Into::into).collect(),
            updated: from.updated.into_iter().map(Into::into).collect(),
            skipped: from.skipped.into_iter().map(Into::into).collect(),
            rejected: from.rejected,
            discarded: from.discarded,
        }
    }
}

/// Predicates for matching URI strings (case-sensitive)
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UriPredicate {
    Prefix(String),
    Exact(String),
}

impl From<UriPredicate> for _repo::UriPredicate {
    fn from(from: UriPredicate) -> Self {
        use _repo::UriPredicate::*;
        match from {
            UriPredicate::Prefix(from) => Prefix(from),
            UriPredicate::Exact(from) => Exact(from),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct UriRelocation {
    pub predicate: UriPredicate,
    pub replacement: String,
}

impl From<UriRelocation> for _repo::UriRelocation {
    fn from(from: UriRelocation) -> Self {
        Self {
            predicate: from.predicate.into(),
            replacement: from.replacement,
        }
    }
}

#[allow(missing_debug_implementations)]
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
        let body_data =
            json::serialize_entity_body_data(&new_track).map_err(warp::reject::custom)?;
        let hdr =
            create_track(&self.db, new_track.into(), body_data).map_err(warp::reject::custom)?;
        Ok(warp::reply::with_status(
            warp::reply::json(&_serde::EntityHeader::from(hdr)),
            StatusCode::CREATED,
        ))
    }

    pub fn handle_update(
        &self,
        uid: EntityUid,
        entity: _serde::Entity,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        let json_data =
            json::serialize_entity_body_data(&entity.1).map_err(warp::reject::custom)?;
        let entity = Entity::from(entity);
        if uid != entity.hdr.uid {
            return Err(warp::reject::custom(anyhow!(
                "Mismatching UIDs: {} <> {}",
                uid,
                entity.hdr.uid,
            )));
        }
        let update_result =
            update_track(&self.db, entity, json_data).map_err(warp::reject::custom)?;
        if let EntityRevisionUpdateResult::Updated(_, next_rev) = update_result {
            let hdr = EntityHeader { uid, rev: next_rev };
            Ok(warp::reply::json(&_serde::EntityHeader::from(hdr)))
        } else {
            Err(warp::reject::custom(anyhow!(
                "Entity not found or revision conflict"
            )))
        }
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
                Some(entity_data) => {
                    let json_data =
                        json::load_entity_data_blob(entity_data).map_err(warp::reject::custom)?;
                    Ok(json::reply_with_content_type(json_data))
                }
                None => Err(warp::reject::not_found()),
            })
    }

    pub fn handle_list(
        &self,
        query_params: TracksQueryParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        let (collection_uid, pagination) = query_params.into();
        list_tracks(&self.db, collection_uid, pagination)
            .and_then(|reply| json::load_entity_data_array_blob(reply.into_iter()))
            .map(json::reply_with_content_type)
            .map_err(warp::reject::custom)
    }

    pub fn handle_search(
        &self,
        query_params: TracksQueryParams,
        search_params: SearchParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        let (collection_uid, pagination) = query_params.into();
        search_tracks(&self.db, collection_uid, pagination, search_params.into())
            .and_then(|reply| json::load_entity_data_array_blob(reply.into_iter()))
            .map(json::reply_with_content_type)
            .map_err(warp::reject::custom)
    }

    pub fn handle_locate(
        &self,
        query_params: TracksQueryParams,
        locate_params: LocateParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        let (collection_uid, pagination) = query_params.into();
        locate_tracks(&self.db, collection_uid, pagination, locate_params.into())
            .and_then(|reply| json::load_entity_data_array_blob(reply.into_iter()))
            .map(json::reply_with_content_type)
            .map_err(warp::reject::custom)
    }

    pub fn handle_replace(
        &self,
        query_params: TracksQueryParams,
        replace_params: ReplaceTracksParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        let (collection_uid, _) = query_params.into();
        let mode = replace_params.mode.into();
        replace_tracks(
            &self.db,
            collection_uid,
            mode,
            replace_params.replacements.into_iter().map(Into::into),
        )
        .map(|val| warp::reply::json(&ReplacedTracks::from(val)))
        .map_err(|err| {
            log::warn!("Failed to replace tracks: {}", err);
            warp::reject::custom(err)
        })
    }

    pub fn handle_purge(
        &self,
        query_params: TracksQueryParams,
        uri_predicates: Vec<UriPredicate>,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        let (collection_uid, _) = query_params.into();
        let uri_predicates = uri_predicates.into_iter().map(Into::into);
        purge_tracks(&self.db, collection_uid, uri_predicates)
            .map(|()| StatusCode::NO_CONTENT)
            .map_err(warp::reject::custom)
    }

    pub fn handle_relocate(
        &self,
        query_params: TracksQueryParams,
        uri_relocations: impl IntoIterator<Item = UriRelocation>,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        let (collection_uid, _) = query_params.into();
        relocate_tracks(
            &self.db,
            collection_uid,
            uri_relocations.into_iter().map(Into::into),
        )
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(warp::reject::custom)
    }

    pub fn handle_albums_count_tracks(
        &self,
        query_params: TracksQueryParams,
        count_params: CountTracksByAlbumParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        let (collection_uid, pagination) = query_params.into();
        count_tracks_by_album(&self.db, collection_uid, pagination, &count_params.into())
            .map(|res| {
                warp::reply::json(
                    &res.into_iter()
                        .map(AlbumTrackCount::from)
                        .collect::<Vec<_>>(),
                )
            })
            .map_err(warp::reject::custom)
    }

    pub fn handle_tags_count_tracks(
        &self,
        query_params: TracksQueryParams,
        count_params: TagCountParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        let (collection_uid, pagination) = query_params.into();
        count_tracks_by_tag(&self.db, collection_uid, pagination, count_params.into())
            .map(|res| {
                warp::reply::json(
                    &res.into_iter()
                        .map(TagAvgScoreCount::from)
                        .collect::<Vec<_>>(),
                )
            })
            .map_err(warp::reject::custom)
    }

    pub fn handle_tags_facets_count_tracks(
        &self,
        query_params: TracksQueryParams,
        count_params: TagFacetCountParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        let (collection_uid, pagination) = query_params.into();
        count_tracks_by_tag_facet(&self.db, collection_uid, pagination, count_params.into())
            .map(|res| {
                warp::reply::json(&res.into_iter().map(TagFacetCount::from).collect::<Vec<_>>())
            })
            .map_err(warp::reject::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencode_tracks_query_params() {
        let collection_uid =
            EntityUid::decode_from_str("DNGwV8sS9XS2GAxfEvgW2NMFxDHwi81CC").unwrap();

        let query = TracksQueryParams {
            collection_uid: Some(collection_uid.clone().into()),
            offset: Some(0),
            limit: Some(2),
        };
        let query_urlencoded = "collectionUid=DNGwV8sS9XS2GAxfEvgW2NMFxDHwi81CC&offset=0&limit=2";
        assert_eq!(
            query_urlencoded,
            serde_urlencoded::to_string(&query).unwrap()
        );
        assert_eq!(
            (
                Some(collection_uid.clone()),
                Pagination {
                    offset: query.offset,
                    limit: query.limit
                }
            ),
            serde_urlencoded::from_str::<TracksQueryParams>(query_urlencoded)
                .unwrap()
                .into()
        );

        let query = TracksQueryParams {
            collection_uid: Some(collection_uid.clone().into()),
            offset: None,
            limit: Some(2),
        };
        let query_urlencoded = "collectionUid=DNGwV8sS9XS2GAxfEvgW2NMFxDHwi81CC&limit=2";
        assert_eq!(
            query_urlencoded,
            serde_urlencoded::to_string(&query).unwrap()
        );
        assert_eq!(
            (
                Some(collection_uid.clone()),
                Pagination {
                    offset: query.offset,
                    limit: query.limit
                }
            ),
            serde_urlencoded::from_str::<TracksQueryParams>(query_urlencoded)
                .unwrap()
                .into()
        );

        let query = TracksQueryParams {
            collection_uid: Some(collection_uid.clone().into()),
            offset: None,
            limit: None,
        };
        let query_urlencoded = "collectionUid=DNGwV8sS9XS2GAxfEvgW2NMFxDHwi81CC";
        assert_eq!(
            query_urlencoded,
            serde_urlencoded::to_string(&query).unwrap()
        );
        assert_eq!(
            (
                Some(collection_uid),
                Pagination {
                    offset: query.offset,
                    limit: query.limit
                }
            ),
            serde_urlencoded::from_str::<TracksQueryParams>(query_urlencoded)
                .unwrap()
                .into()
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
            (
                None,
                Pagination {
                    offset: query.offset,
                    limit: query.limit
                }
            ),
            serde_urlencoded::from_str::<TracksQueryParams>(query_urlencoded)
                .unwrap()
                .into()
        );
    }
}
