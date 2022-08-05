// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::num::NonZeroUsize;

use bitflags::bitflags;

use aoide_core::{
    audio::DurationMs,
    media::content::ContentMetadata,
    track::{Entity as TrackEntity, Track},
};

use aoide_core_api::track::search::*;

use aoide_repo::{
    collection::RecordId as CollectionId,
    track::{CollectionRepo as TrackCollectionRepo, RecordHeader, RecordId as TrackId},
};

use super::*;

bitflags! {
    /// A bitmask for controlling how and if content metadata is
    /// re-imported from the source.
    pub struct SearchFlags: u8 {
        const NONE           = 0b0000_0000; // least restrictive
        const SOURCE_TRACKED   = 0b0000_0001;
        const ALBUM_ARTIST     = 0b0000_0010;
        const ALBUM_TITLE      = 0b0000_0100;
        const TRACK_ARTIST     = 0b0000_1000;
        const TRACK_TITLE      = 0b0001_0000;
        const RECORDED_AT      = 0b0010_0000;
        const RELEASED_AT      = 0b0100_0000;
        const RELEASED_ORIG_AT = 0b1000_0000;
        const ALL              = 0b1111_1111; // most restrictive
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    pub audio_duration_tolerance: DurationMs,
    pub max_results: NonZeroUsize,
    pub search_flags: SearchFlags,
}

impl Params {
    #[allow(unsafe_code)]
    #[must_use]
    pub const fn new() -> Params {
        // More than one result is necessary to decide if it is unambiguous
        let max_results = unsafe { NonZeroUsize::new_unchecked(2) };
        Self {
            audio_duration_tolerance: DurationMs::from_inner(500.0), // +/- 500 ms
            max_results,
            search_flags: SearchFlags::ALL,
        }
    }

    #[must_use]
    pub const fn with_max_results(max_results: NonZeroUsize) -> Params {
        Self {
            max_results,
            ..Self::new()
        }
    }
}

impl Default for Params {
    fn default() -> Self {
        Self::new()
    }
}

pub fn find_duplicates<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    track_id: TrackId,
    track: Track,
    params: &Params,
) -> RepoResult<Vec<(TrackId, TrackEntity)>>
where
    Repo: TrackCollectionRepo,
{
    let Params {
        audio_duration_tolerance,
        search_flags,
        max_results,
    } = params;
    let mut all_filters = Vec::with_capacity(10);
    if search_flags.contains(SearchFlags::TRACK_ARTIST) {
        if let Some(track_artist) = track.track_artist() {
            let track_artist = track_artist.trim();
            if !track_artist.is_empty() {
                all_filters.push(Filter::Phrase(PhraseFieldFilter {
                    fields: vec![StringField::TrackArtist],
                    terms: vec![track_artist.to_owned()],
                }));
            }
        }
    }
    if search_flags.contains(SearchFlags::TRACK_TITLE) {
        if let Some(track_title) = track.track_title() {
            let track_title = track_title.trim();
            if !track_title.is_empty() {
                all_filters.push(Filter::Phrase(PhraseFieldFilter {
                    fields: vec![StringField::TrackTitle],
                    terms: vec![track_title.to_owned()],
                }));
            }
        }
    }
    if search_flags.contains(SearchFlags::ALBUM_ARTIST) {
        if let Some(album_artist) = track.album_artist() {
            let album_artist = album_artist.trim();
            if !album_artist.is_empty() {
                all_filters.push(Filter::Phrase(PhraseFieldFilter {
                    fields: vec![StringField::AlbumArtist],
                    terms: vec![album_artist.to_owned()],
                }));
            }
        }
    }
    if search_flags.contains(SearchFlags::ALBUM_TITLE) {
        if let Some(album_title) = track.album_title() {
            let album_title = album_title.trim();
            if !album_title.is_empty() {
                all_filters.push(Filter::Phrase(PhraseFieldFilter {
                    fields: vec![StringField::AlbumTitle],
                    terms: vec![album_title.to_owned()],
                }));
            }
        }
    }
    if search_flags.contains(SearchFlags::RECORDED_AT) {
        all_filters.push(if let Some(recorded_at) = track.recorded_at {
            Filter::recorded_at_equals(recorded_at)
        } else {
            Filter::DateTime(DateTimeFieldFilter {
                field: DateTimeField::RecordedAt,
                predicate: DateTimePredicate::Equal(None),
            })
        });
    }
    if search_flags.contains(SearchFlags::RELEASED_AT) {
        all_filters.push(if let Some(released_at) = track.released_at {
            Filter::released_at_equals(released_at)
        } else {
            Filter::DateTime(DateTimeFieldFilter {
                field: DateTimeField::ReleasedAt,
                predicate: DateTimePredicate::Equal(None),
            })
        });
    }
    if search_flags.contains(SearchFlags::RELEASED_ORIG_AT) {
        all_filters.push(if let Some(released_orig_at) = track.released_orig_at {
            Filter::released_at_equals(released_orig_at)
        } else {
            Filter::DateTime(DateTimeFieldFilter {
                field: DateTimeField::ReleasedOrigAt,
                predicate: DateTimePredicate::Equal(None),
            })
        });
    }
    if search_flags.contains(SearchFlags::SOURCE_TRACKED) {
        all_filters.push(Filter::Condition(ConditionFilter::SourceTracked));
    }
    // Only sources with similar audio duration
    let audio_duration_ms = match track.media_source.content_metadata {
        ContentMetadata::Audio(content) => content.duration,
    };
    all_filters.push(if let Some(audio_duration_ms) = audio_duration_ms {
        Filter::audio_duration_around(audio_duration_ms, *audio_duration_tolerance)
    } else {
        Filter::Numeric(NumericFieldFilter {
            field: NumericField::AudioDurationMs,
            predicate: NumericPredicate::Equal(None),
        })
    });
    // Only sources with equal content/file type
    all_filters.push(Filter::Phrase(PhraseFieldFilter {
        fields: vec![StringField::ContentType],
        terms: vec![track.media_source.content_type.to_string()],
    }));
    let filter = Filter::All(all_filters);
    // Prefer recently added sources, e.g. after scanning the file system
    let ordering = vec![SortOrder {
        field: SortField::CollectedAt,
        direction: SortDirection::Descending,
    }];
    let mut candidates = Vec::new();
    repo.search_tracks(
        collection_id,
        &Default::default(),
        Some(filter),
        ordering,
        &mut candidates,
    )?;
    Ok(candidates
        .into_iter()
        .filter_map(|(record_header, entity)| {
            if record_header.id == track_id {
                // Exclude the track if contained in the search results
                None
            } else {
                Some((record_header.id, entity))
            }
        })
        .take(max_results.get())
        .collect())
}

pub fn find_duplicate_by_media_source_content_path<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    content_path: &str,
    params: &Params,
) -> RepoResult<Vec<(TrackId, TrackEntity)>>
where
    Repo: TrackCollectionRepo,
{
    let (_media_source_id, RecordHeader { id: track_id, .. }, entity) =
        repo.load_track_entity_by_media_source_content_path(collection_id, content_path)?;
    find_duplicates(repo, collection_id, track_id, entity.raw.body.track, params)
}
