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
    audio::DurationMs,
    entity::{EntityHeader, EntityRevision, EntityUid},
    track::{release::DateOrDateTime, Entity, Track},
    util::clock::DateTime,
};

use semval::IsValid;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StringField {
    AlbumArtist,
    AlbumTitle,
    SourceType, // RFC 6838 media type
    SourcePath, // RFC 3986 percent-encoded URI
    TrackArtist,
    TrackComposer,
    TrackTitle,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NumericField {
    AudioBitrateBps,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DateTimeField {
    LastPlayedAt,
    ReleasedAt,
    SourceCollectedAt,
    SourceSynchronizedAt,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ConditionFilter {
    SourceTracked,
    SourceUntracked,
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
    pub path: StringPredicateBorrowed<'s>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SortField {
    AlbumArtist,
    AlbumTitle,
    AudioBitrateBps,
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
    SourceType,
    SourcePath,
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
    Condition(ConditionFilter),
    Tag(tag::Filter),
    CueLabel(StringFilter),
    PlaylistUid(EntityUid),
    All(Vec<SearchFilter>),
    Any(Vec<SearchFilter>),
    Not(Box<SearchFilter>),
}

impl SearchFilter {
    pub fn released_at_equals(released_at: DateOrDateTime) -> Self {
        match released_at {
            DateOrDateTime::DateTime(released_at) => Self::DateTime(DateTimeFieldFilter {
                field: DateTimeField::ReleasedAt,
                predicate: DateTimePredicate::Equal(Some(released_at)),
            }),
            DateOrDateTime::Date(date) => Self::Numeric(NumericFieldFilter {
                field: NumericField::ReleaseDate,
                predicate: NumericPredicate::Equal(Some(date.to_inner().into())),
            }),
        }
    }

    pub fn audio_duration_around(duration: DurationMs, epsilon: DurationMs) -> Self {
        debug_assert!(duration.is_valid());
        debug_assert!(epsilon.is_valid());
        let duration_value = duration.to_inner();
        let epsilon_value = epsilon.to_inner();
        if epsilon_value > 0.0 {
            Self::All(vec![
                Self::Numeric(NumericFieldFilter {
                    field: NumericField::AudioDurationMs,
                    predicate: NumericPredicate::GreaterOrEqual(
                        (duration_value - epsilon_value).max(0.0),
                    ),
                }),
                Self::Numeric(NumericFieldFilter {
                    field: NumericField::AudioDurationMs,
                    predicate: NumericPredicate::LessOrEqual(duration_value + epsilon_value),
                }),
            ])
        } else {
            Self::Numeric(NumericFieldFilter {
                field: NumericField::AudioDurationMs,
                predicate: NumericPredicate::Equal(Some(duration_value)),
            })
        }
    }
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
    Created(MediaSourceId, RecordId, Entity),
    Updated(MediaSourceId, RecordId, Entity),
    Unchanged(MediaSourceId, RecordId, Entity),
    NotCreated(Track),
    NotUpdated(MediaSourceId, RecordId, Track),
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

    fn load_track_entity_by_media_source_path(
        &self,
        collection_id: CollectionId,
        media_source_path: &str,
    ) -> RepoResult<(MediaSourceId, RecordHeader, Entity)>;

    fn resolve_track_entity_header_by_media_source_path(
        &self,
        collection_id: CollectionId,
        media_source_path: &str,
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

    fn replace_collected_track_by_media_source_path(
        &self,
        collection_id: CollectionId,
        preserve_collected_at: bool,
        replace_mode: ReplaceMode,
        track: Track,
    ) -> RepoResult<ReplaceOutcome>;

    fn purge_tracks_by_media_source_media_source_path_predicate(
        &self,
        collection_id: CollectionId,
        media_source_path_predicate: StringPredicateBorrowed<'_>,
    ) -> RepoResult<usize>;

    fn search_collected_tracks(
        &self,
        collection_id: CollectionId,
        pagination: &Pagination,
        filter: Option<SearchFilter>,
        ordering: Vec<SortOrder>,
        collector: &mut dyn ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
    ) -> RepoResult<usize>;

    fn count_collected_tracks(&self, collection_id: CollectionId) -> RepoResult<u64>;
}
