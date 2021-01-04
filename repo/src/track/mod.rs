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

use crate::{
    collection::RecordId as CollectionId,
    media::source::{RecordId as MediaSourceId, Repo as MediaSourceRepo},
    prelude::*,
    tag,
};

record_id_newtype!(RecordId);

pub type RecordHeader = crate::RecordHeader<RecordId>;

use aoide_core::{
    entity::{EntityHeader, EntityRevision, EntityUid},
    track::{Entity, Track},
    util::clock::DateTime,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StringField {
    AlbumArtist,
    AlbumTitle,
    MediaType,
    SourceUri, // percent-encoded URI
    SourceUriDecoded,
    TrackArtist,
    TrackComposer,
    TrackTitle,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NumericField {
    AudioBitRateBps,
    AudioChannelCount,
    AudioDurationMs,
    AudioLoudnessLufs,
    AudioSampleRateHz,
    DiscNumber,
    DiscTotal,
    MusicTempoBpm,
    MusicKeyCode,
    ReleaseDate,
    TrackNumber,
    TrackTotal,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DateTimeField {
    LastPlayedAt,
    ReleasedAt,
    SourceCollectedAt,
    SourceSynchronizedAt,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ScalarFieldFilter<F, V> {
    pub field: F,
    pub predicate: ScalarPredicate<V>,
}

pub type NumericFieldFilter = ScalarFieldFilter<NumericField, NumericValue>;

pub type DateTimeFieldFilter = ScalarFieldFilter<DateTimeField, DateTime>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhraseFieldFilter {
    // Empty == All available string fields are considered
    // Disjunction, i.e. a match in one of the fields is sufficient
    pub fields: Vec<StringField>,

    // Concatenated with wildcards and filtered using
    // case-insensitive "contains" semantics against each
    // of the selected fields, e.g. ["pa", "la", "bell"]
    // ["tt, ll"] will both match "Patti LaBelle". An empty
    // argument matches empty as well as missing/null fields.
    pub terms: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFilterBorrowed<'s> {
    pub uri: StringPredicateBorrowed<'s>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SortField {
    AlbumArtist,
    AlbumTitle,
    AudioBitRateBps,
    AudioChannelCount,
    AudioDurationMs,
    AudioLoudnessLufs,
    AudioSampleRateHz,
    CreatedAt,
    DiscNumber,
    DiscTotal,
    LastPlayedAt,
    MusicTempoBpm,
    MusicKeyCode,
    ReleaseDate,
    SourceCollectedAt,
    SourceSynchronizedAt,
    SourceUri,
    SourceUriDecoded,
    TimesPlayed,
    TrackArtist,
    TrackNumber,
    TrackTitle,
    TrackTotal,
    UpdatedAt,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SortOrder {
    pub field: SortField,
    pub direction: SortDirection,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SearchFilter {
    Phrase(PhraseFieldFilter),
    Numeric(NumericFieldFilter),
    DateTime(DateTimeFieldFilter),
    Tag(tag::Filter),
    CueLabel(StringFilter),
    All(Vec<SearchFilter>),
    Any(Vec<SearchFilter>),
    Not(Box<SearchFilter>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchParams {
    pub filter: Option<SearchFilter>,
    pub ordering: Vec<SortOrder>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringFieldCounts {
    pub field: StringField,
    pub counts: Vec<StringCount>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplaceMode {
    CreateOnly,
    UpdateOnly,
    UpdateOrCreate,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReplaceOutcome {
    NotCreated(Track),
    NotUpdated(RecordId, Track),
    Created(RecordId, Entity),
    Updated(RecordId, Entity),
    Unchanged(RecordId, Entity),
}

pub trait EntityRepo: MediaSourceRepo {
    fn resolve_track_id(&self, uid: &EntityUid) -> RepoResult<RecordId> {
        self.resolve_track_entity_revision(uid)
            .map(|(hdr, _rev)| hdr.id)
    }

    fn resolve_track_entity_revision(
        &self,
        uid: &EntityUid,
    ) -> RepoResult<(RecordHeader, EntityRevision)>;

    fn load_track_entity(&self, id: RecordId) -> RepoResult<(RecordHeader, Entity)>;

    fn load_track_entity_by_uid(&self, uid: &EntityUid) -> RepoResult<(RecordHeader, Entity)>;

    fn load_track_entity_by_media_source_uri(
        &self,
        collection_id: CollectionId,
        uri: &str,
    ) -> RepoResult<(MediaSourceId, RecordHeader, Entity)>;

    fn resolve_track_entity_header_by_media_source_uri(
        &self,
        collection_id: CollectionId,
        uri: &str,
    ) -> RepoResult<(MediaSourceId, RecordHeader, EntityHeader)>;

    fn list_track_entities(
        &self,
        pagination: &Pagination,
    ) -> RepoResult<Vec<(RecordHeader, Entity)>>;

    fn insert_track_entity(
        &self,
        created_at: DateTime,
        media_source_id: MediaSourceId,
        created_entity: &Entity,
    ) -> RepoResult<RecordId>;

    fn update_track_entity(
        &self,
        id: RecordId,
        updated_at: DateTime,
        media_source_id: MediaSourceId,
        updated_entity: &Entity,
    ) -> RepoResult<()>;

    fn delete_track_entity(&self, id: RecordId) -> RepoResult<()>;

    fn replace_collected_track_by_media_source_uri(
        &self,
        collection_id: CollectionId,
        replace_mode: ReplaceMode,
        track: Track,
    ) -> RepoResult<ReplaceOutcome>;

    fn purge_tracks_by_media_source_uri_predicate(
        &self,
        collection_id: CollectionId,
        uri_predicate: StringPredicateBorrowed<'_>,
    ) -> RepoResult<usize>;

    fn search_collected_tracks(
        &self,
        collection_id: CollectionId,
        pagination: &Pagination,
        filter: Option<SearchFilter>,
        ordering: Vec<SortOrder>,
        collector: &mut dyn ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
    ) -> RepoResult<()>;

    fn count_collected_tracks(&self, collection_id: CollectionId) -> RepoResult<u64>;
}
