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
    pub use aoide_core::{
        entity::{EntityHeader, EntityRevision, EntityUid},
        track::{Entity, Track},
    };
}

mod _repo {
    pub use aoide_repo::{
        entity::{EntityBodyData, EntityData, EntityDataFormat, EntityDataVersion},
        tag::{
            AvgScoreCount as TagAvgScoreCount, CountParams as TagCountParams,
            FacetCount as TagFacetCount, FacetCountParams as TagFacetCountParams,
            Filter as TagFilter, SortField as TagSortField, SortOrder as TagSortOrder,
        },
        track::{
            AlbumCountParams, AlbumCountResults, LocateParams, NumericField, NumericFieldFilter,
            PhraseFieldFilter, ReplaceMode, ReplaceResult, SearchFilter, SearchParams, SortField,
            SortOrder, StringField,
        },
        util::{UriPredicate, UriRelocation},
        FilterModifier, NumericPredicate, NumericValue, SortDirection, StringFilter,
        StringPredicate,
    };
}

use aoide_core::{tag::ScoreValue as TagScoreValue, track::release::ReleaseYear};

use aoide_repo::{
    track::{Albums as _, Repo as _, Tags as _},
    Pagination, PaginationLimit, PaginationOffset, RepoResult,
};

use aoide_core_serde::{
    entity::{EntityHeader, EntityUid},
    track::{Entity, Track},
};

use aoide_repo_sqlite::track::Repository;

use futures::future::{self, Future};
use std::io::Write;
use warp::http::StatusCode;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TracksQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_uid: Option<EntityUid>,

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

impl From<TracksQueryParams> for (Option<_core::EntityUid>, Pagination) {
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
    ReleaseYear,
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
            SortField::ReleaseYear => ReleaseYear,
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
    ReleaseYear,
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
            NumericField::ReleaseYear => ReleaseYear,
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
pub struct AlbumCountParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_release_year: Option<ReleaseYear>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_release_year: Option<ReleaseYear>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<SortOrder>,
}

impl From<AlbumCountParams> for _repo::AlbumCountParams {
    fn from(from: AlbumCountParams) -> Self {
        Self {
            min_release_year: from.min_release_year,
            max_release_year: from.max_release_year,
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

    #[serde(rename = "y", skip_serializing_if = "Option::is_none")]
    pub release_year: Option<ReleaseYear>,

    #[serde(rename = "n")]
    pub total_count: usize,
}

impl From<_repo::AlbumCountResults> for AlbumTrackCount {
    fn from(from: _repo::AlbumCountResults) -> Self {
        Self {
            title: from.title,
            artist: from.artist,
            release_year: from.release_year,
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
pub struct TrackReplacement {
    // The URI for locating any existing track that is supposed
    // to replaced by the provided track.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    media_uri: String,

    track: Track,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReplaceTracksParams {
    mode: ReplaceMode,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    replacements: Vec<TrackReplacement>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReplacedTracks {
    pub created: Vec<EntityHeader>,
    pub updated: Vec<EntityHeader>,
    pub skipped: Vec<EntityHeader>,
    pub rejected: Vec<String>,  // e.g. ambiguous or inconsistent
    pub discarded: Vec<String>, // e.g. nonexistent and need to be created
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

const ENTITY_DATA_FORMAT: _repo::EntityDataFormat = _repo::EntityDataFormat::JSON;

const ENTITY_DATA_VERSION: _repo::EntityDataVersion =
    _repo::EntityDataVersion { major: 0, minor: 0 };

fn write_json_body_data(track: &Track) -> Fallible<_repo::EntityBodyData> {
    Ok((
        ENTITY_DATA_FORMAT,
        ENTITY_DATA_VERSION,
        serde_json::to_vec(track)?,
    ))
}

fn read_json_entity(entity_data: _repo::EntityData) -> Fallible<_core::Entity> {
    let (hdr, json_data) = load_json_entity_data(entity_data)?;
    let track: Track = serde_json::from_slice(&json_data)?;
    Ok(_core::Entity::new(hdr, _core::Track::from(track)))
}

fn load_json_entity_data(
    entity_data: _repo::EntityData,
) -> Fallible<(_core::EntityHeader, Vec<u8>)> {
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

fn load_and_write_entity_data_json(
    mut json_writer: &mut impl Write,
    entity_data: _repo::EntityData,
) -> Fallible<()> {
    let (hdr, json_data) = load_json_entity_data(entity_data)?;
    json_writer.write_all(b"[")?;
    serde_json::to_writer(&mut json_writer, &EntityHeader::from(hdr))?;
    json_writer.write_all(b",")?;
    json_writer.write_all(&json_data)?;
    json_writer.write_all(b"]")?;
    Ok(())
}

fn entity_data_json_size(entity_data: &_repo::EntityData) -> usize {
    let uid_bytes = 33;
    let rev_ver_bytes = ((entity_data.0).rev.ver as f64).log10().ceil() as usize;
    let rev_ts_bytes = 16;
    // ["<uid>",[<rev.ver>,<rev.ts>]]
    (entity_data.1).2.len() + uid_bytes + rev_ver_bytes + rev_ts_bytes + 8
}

fn load_entity_data_into_json(entity_data: _repo::EntityData) -> Fallible<Vec<u8>> {
    let mut json_writer = Vec::with_capacity(entity_data_json_size(&entity_data));
    load_and_write_entity_data_json(&mut json_writer, entity_data)?;
    Ok(json_writer)
}

fn load_entity_data_iter_into_json_array(
    entity_data_iter: impl Iterator<Item = _repo::EntityData> + Clone,
) -> Fallible<Vec<u8>> {
    let mut json_writer = Vec::with_capacity(entity_data_iter.clone().fold(
        /*closing bracket*/ 1,
        |acc, ref entity_data| {
            acc + entity_data_json_size(&entity_data) + /*opening bracket or comma*/ 1
        },
    ));
    json_writer.write_all(b"[")?;
    for (i, entity_data) in entity_data_iter.enumerate() {
        if i > 0 {
            json_writer.write_all(b",")?;
        }
        load_and_write_entity_data_json(&mut json_writer, entity_data)?;
    }
    json_writer.write_all(b"]")?;
    json_writer.flush()?;
    Ok(json_writer)
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
        let body_data = write_json_body_data(&new_track).map_err(warp::reject::custom)?;
        let hdr =
            create_track(&self.db, new_track.into(), body_data).map_err(warp::reject::custom)?;
        Ok(warp::reply::with_status(
            warp::reply::json(&EntityHeader::from(hdr)),
            StatusCode::CREATED,
        ))
    }

    pub fn handle_update(
        &self,
        uid: _core::EntityUid,
        entity: Entity,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        let json_data = write_json_body_data(&entity.1).map_err(warp::reject::custom)?;
        let entity = _core::Entity::from(entity);
        if uid != entity.hdr.uid {
            return Err(warp::reject::custom(failure::format_err!(
                "Mismatching UIDs: {} <> {}",
                uid,
                entity.hdr.uid,
            )));
        }
        let (_, next_rev) =
            update_track(&self.db, entity, json_data).map_err(warp::reject::custom)?;
        if let Some(rev) = next_rev {
            let hdr = _core::EntityHeader { uid, rev };
            Ok(warp::reply::json(&EntityHeader::from(hdr)))
        } else {
            Err(warp::reject::custom(failure::format_err!(
                "Inexistent entity or revision conflict"
            )))
        }
    }

    pub fn handle_delete(
        &self,
        uid: _core::EntityUid,
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

    pub fn handle_load(
        &self,
        uid: _core::EntityUid,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        load_track(&self.db, &uid)
            .map_err(warp::reject::custom)
            .and_then(|res| match res {
                Some(entity_data) => {
                    let json_data =
                        load_entity_data_into_json(entity_data).map_err(warp::reject::custom)?;
                    Ok(reply_with_json_content(json_data))
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
            .and_then(|reply| load_entity_data_iter_into_json_array(reply.into_iter()))
            .map(reply_with_json_content)
            .map_err(warp::reject::custom)
    }

    pub fn handle_search(
        &self,
        query_params: TracksQueryParams,
        search_params: SearchParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        let (collection_uid, pagination) = query_params.into();
        search_tracks(&self.db, collection_uid, pagination, search_params.into())
            .and_then(|reply| load_entity_data_iter_into_json_array(reply.into_iter()))
            .map(reply_with_json_content)
            .map_err(warp::reject::custom)
    }

    pub fn handle_locate(
        &self,
        query_params: TracksQueryParams,
        locate_params: LocateParams,
    ) -> impl Future<Item = impl warp::Reply, Error = warp::reject::Rejection> {
        let (collection_uid, pagination) = query_params.into();
        locate_tracks(&self.db, collection_uid, pagination, locate_params.into())
            .and_then(|reply| load_entity_data_iter_into_json_array(reply.into_iter()))
            .map(reply_with_json_content)
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
            replace_params.replacements.into_iter(),
        )
        .map(|val| warp::reply::json(&val))
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
        count_params: AlbumCountParams,
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

fn create_track(
    db: &SqlitePooledConnection,
    new_track: _core::Track,
    body_data: _repo::EntityBodyData,
) -> RepoResult<_core::EntityHeader> {
    let repository = Repository::new(&*db);
    let hdr = _core::EntityHeader::initial_random();
    let entity = _core::Entity::new(hdr.clone(), new_track);
    db.transaction::<_, Error, _>(|| repository.insert_track(entity, body_data).map(|()| hdr))
}

fn update_track(
    db: &SqlitePooledConnection,
    track: _core::Entity,
    body_data: _repo::EntityBodyData,
) -> RepoResult<(_core::EntityRevision, Option<_core::EntityRevision>)> {
    let repository = Repository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.update_track(track, body_data))
}

fn delete_track(db: &SqlitePooledConnection, uid: &_core::EntityUid) -> RepoResult<Option<()>> {
    let repository = Repository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.delete_track(uid))
}

fn load_track(
    pooled_connection: &SqlitePooledConnection,
    uid: &_core::EntityUid,
) -> RepoResult<Option<_repo::EntityData>> {
    let repository = Repository::new(&*pooled_connection);
    pooled_connection.transaction::<_, Error, _>(|| repository.load_track(uid))
}

fn list_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<_core::EntityUid>,
    pagination: Pagination,
) -> impl Future<Item = Vec<_repo::EntityData>, Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.search_tracks(collection_uid.as_ref(), pagination, Default::default())
    }))
}

fn search_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<_core::EntityUid>,
    pagination: Pagination,
    params: _repo::SearchParams,
) -> impl Future<Item = Vec<_repo::EntityData>, Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.search_tracks(collection_uid.as_ref(), pagination, params)
    }))
}

fn locate_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<_core::EntityUid>,
    pagination: Pagination,
    params: _repo::LocateParams,
) -> impl Future<Item = Vec<_repo::EntityData>, Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.locate_tracks(collection_uid.as_ref(), pagination, params)
    }))
}

fn replace_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<_core::EntityUid>,
    mode: _repo::ReplaceMode,
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
            use _repo::ReplaceResult::*;
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
                    results.skipped.push(hdr.into());
                }
                Created(hdr) => {
                    results.created.push(hdr.into());
                }
                Updated(hdr) => {
                    results.updated.push(hdr.into());
                }
            }
        }
        Ok(results)
    }))
}

fn purge_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<_core::EntityUid>,
    uri_predicates: impl IntoIterator<Item = _repo::UriPredicate>,
) -> impl Future<Item = (), Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        for uri_predicate in uri_predicates {
            use _repo::StringPredicate::*;
            use _repo::UriPredicate::*;
            let locate_params = match &uri_predicate {
                Prefix(media_uri) => _repo::LocateParams {
                    media_uri: StartsWith(media_uri.to_owned()),
                },
                Exact(media_uri) => _repo::LocateParams {
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
                let _core::Entity { hdr, mut body, .. } = read_json_entity(entity)?;
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
                        let entity = _core::Entity::new(hdr, body);
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

fn relocate_tracks(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<_core::EntityUid>,
    uri_relocations: impl IntoIterator<Item = _repo::UriRelocation>,
) -> impl Future<Item = (), Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        for uri_relocation in uri_relocations {
            let locate_params = match &uri_relocation.predicate {
                _repo::UriPredicate::Prefix(uri_prefix) => _repo::LocateParams {
                    media_uri: _repo::StringPredicate::StartsWith(uri_prefix.to_owned()),
                },
                _repo::UriPredicate::Exact(uri) => _repo::LocateParams {
                    media_uri: _repo::StringPredicate::Equals(uri.to_owned()),
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
                    _repo::UriPredicate::Prefix(uri_prefix) => track
                        .relocate_media_source_by_uri_prefix(
                            &uri_prefix,
                            &uri_relocation.replacement,
                        ),
                    _repo::UriPredicate::Exact(uri) => {
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
                    let entity = _core::Entity::new(hdr, track);
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

fn count_tracks_by_album(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<_core::EntityUid>,
    pagination: Pagination,
    params: &_repo::AlbumCountParams,
) -> impl Future<Item = Vec<_repo::AlbumCountResults>, Error = Error> {
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_tracks_by_album(collection_uid.as_ref(), params, pagination)
    }))
}

fn count_tracks_by_tag(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<_core::EntityUid>,
    pagination: Pagination,
    mut params: _repo::TagCountParams,
) -> impl Future<Item = Vec<_repo::TagAvgScoreCount>, Error = Error> {
    params.dedup_facets();
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_tracks_by_tag(collection_uid.as_ref(), &params, pagination)
    }))
}

fn count_tracks_by_tag_facet(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: Option<_core::EntityUid>,
    pagination: Pagination,
    mut params: _repo::TagFacetCountParams,
) -> impl Future<Item = Vec<_repo::TagFacetCount>, Error = Error> {
    params.dedup_facets();
    let repository = Repository::new(&*pooled_connection);
    future::result(pooled_connection.transaction::<_, Error, _>(|| {
        repository.count_tracks_by_tag_facet(collection_uid.as_ref(), &params, pagination)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencode_tracks_query_params() {
        let collection_uid =
            _core::EntityUid::decode_from_str("DNGwV8sS9XS2GAxfEvgW2NMFxDHwi81CC").unwrap();

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
