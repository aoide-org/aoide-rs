// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::RangeBounds;

use bitflags::bitflags;
use jiff::{Timestamp, tz::TimeZone};
use rand::{RngCore, seq::SliceRandom as _};
use semval::prelude::*;

use crate::{
    EntityHeaderTyped, EntityUidInvalidity, EntityUidTyped, TrackUid,
    util::{
        color::{Color, ColorInvalidity},
        random::adhoc_rng,
    },
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SeparatorItem {
    /// Semantic type of the playlist separator
    ///
    /// A custom identifier that allows third-party applications
    /// to distinguish different kinds of playlist separators.
    pub kind: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackItem {
    /// A reference to the track.
    pub uid: TrackUid,
}

#[derive(Copy, Clone, Debug)]
pub enum TrackItemInvalidity {
    Uid(EntityUidInvalidity),
}

impl Validate for TrackItem {
    type Invalidity = TrackItemInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.uid, Self::Invalidity::Uid)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Separator(SeparatorItem),
    Track(TrackItem),
    // TODO: Add more items like an optional transition between
    // two subsequent track items?
    //Transition(transition::Item),
}

impl Item {
    #[must_use]
    pub const fn is_separator(&self) -> bool {
        matches!(self, Self::Separator(_))
    }

    #[must_use]
    pub const fn is_track(&self) -> bool {
        matches!(self, Self::Track(_))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ItemInvalidity {
    Track(TrackItemInvalidity),
}

impl Validate for Item {
    type Invalidity = ItemInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        match self {
            Item::Separator(_) => context,
            Item::Track(track) => context.validate_with(track, Self::Invalidity::Track),
        }
        .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Timestamp when this entry has been created and added to the playlist.
    pub added_ts: Timestamp,

    /// Optional title for display.
    pub title: Option<String>,

    /// Optional personal notes.
    pub notes: Option<String>,

    /// The actual item, currently just a reference to a single track.
    pub item: Item,
}

#[derive(Copy, Clone, Debug)]
pub enum EntryInvalidity {
    TitleEmpty,
    Item(ItemInvalidity),
}

impl Validate for Entry {
    type Invalidity = EntryInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self { title, item, .. } = self;
        ValidationContext::new()
            .invalidate_if(
                title.as_ref().is_some_and(|title| title.trim().is_empty()),
                Self::Invalidity::TitleEmpty,
            )
            .validate_with(item, Self::Invalidity::Item)
            .into()
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Flags: u8 {
        const LOCKED = 0b0000_0001;
    }
}

impl Flags {
    #[must_use]
    pub const fn is_valid(self) -> bool {
        Self::all().contains(self)
    }
}

impl Default for Flags {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct FlagsInvalidity;

impl Validate for Flags {
    type Invalidity = FlagsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!Flags::is_valid(*self), FlagsInvalidity)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Playlist {
    /// Mandatory name.
    pub title: String,

    /// Semantic type of the playlist
    ///
    /// A custom identifier that allows third-party applications
    /// to distinguish different kinds of playlists for different
    /// purposes and depending on their use case, e.g. generated
    /// session or history playlists for logging all tracks that
    /// have been played during this session.
    pub kind: Option<String>,

    /// Optional notes.
    pub notes: Option<String>,

    /// Optional color for display purposes.
    pub color: Option<Color>,

    /// Optional time zone.
    ///
    /// Needed for deriving wall-clock times from timestamps. Useful
    /// to reconstruct the original, local time for history playlists.
    ///
    /// All entries share a common time zone.
    pub time_zone: Option<TimeZone>,

    pub flags: Flags,
}

#[derive(Copy, Clone, Debug)]
pub enum PlaylistInvalidity {
    TitleEmpty,
    KindEmpty,
    Color(ColorInvalidity),
    Flags(FlagsInvalidity),
}

impl Validate for Playlist {
    type Invalidity = PlaylistInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self {
            title, kind, color, ..
        } = self;
        ValidationContext::new()
            .invalidate_if(title.trim().is_empty(), Self::Invalidity::TitleEmpty)
            .invalidate_if(
                kind.as_ref().is_some_and(|kind| kind.trim().is_empty()),
                Self::Invalidity::KindEmpty,
            )
            .validate_with(color, Self::Invalidity::Color)
            .validate_with(&self.flags, PlaylistInvalidity::Flags)
            .into()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EntityType;

pub type EntityUid = EntityUidTyped<EntityType>;

pub type EntityHeader = EntityHeaderTyped<EntityType>;

pub type Entity = crate::entity::Entity<EntityType, Playlist, PlaylistInvalidity>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistWithEntries {
    pub playlist: Playlist,

    /// Ordered list of playlist entries.
    pub entries: Vec<Entry>,
}

impl PlaylistWithEntries {
    #[must_use]
    pub fn entries_added_ts_minmax(&self) -> Option<(Timestamp, Timestamp)> {
        let mut entries = self.entries.iter();
        if let Some(first_added) = entries.next().map(|e| &e.added_ts) {
            let mut added_min = *first_added;
            let mut added_max = *first_added;
            for e in entries {
                if added_min > e.added_ts {
                    added_min = e.added_ts;
                }
                if added_max < e.added_ts {
                    added_max = e.added_ts;
                }
            }
            Some((added_min, added_max))
        } else {
            None
        }
    }

    pub fn append_entries(&mut self, new_entries: impl IntoIterator<Item = Entry>) {
        self.replace_entries(self.entries.len().., new_entries);
    }

    pub fn insert_entries(&mut self, before: usize, new_entries: impl IntoIterator<Item = Entry>) {
        self.replace_entries(before..before, new_entries);
    }

    pub fn replace_entries(
        &mut self,
        range: impl RangeBounds<usize>,
        new_entries: impl IntoIterator<Item = Entry>,
    ) {
        self.entries.splice(range, new_entries);
    }

    pub fn remove_entries(&mut self, range: impl RangeBounds<usize>) {
        self.entries.drain(range);
    }

    pub fn remove_all_entries(&mut self) {
        self.entries.clear();
    }

    pub fn shuffle_entries(&mut self) {
        self.shuffle_entries_with(&mut adhoc_rng());
    }

    pub fn shuffle_entries_with<T: RngCore>(&mut self, rng: &mut T) {
        self.entries.shuffle(rng);
    }

    // Sort entries by their creation time stamp, preserving the
    // order of entries with equal time stamps.
    pub fn reverse_entries(&mut self) {
        self.entries.reverse();
    }

    // Sort entries by their creation time stamp, preserving the
    // order of entries with equal time stamps.
    pub fn sort_entries_chronologically(&mut self) {
        self.entries.sort_by_key(|entry| entry.added_ts);
    }

    #[must_use]
    pub fn count_tracks(&self) -> usize {
        self.entries.iter().filter(|e| e.item.is_track()).count()
    }

    #[must_use]
    pub fn count_distinct_tracks(&self) -> usize {
        let mut uids = self
            .entries
            .iter()
            .filter_map(|e| match &e.item {
                Item::Track(track) => Some(&track.uid),
                Item::Separator(_) => None,
            })
            .collect::<Vec<_>>();
        uids.sort_unstable();
        uids.dedup();
        uids.len()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PlaylistWithEntriesInvalidity {
    Playlist(PlaylistInvalidity),
    Entry(usize, EntryInvalidity),
}

impl Validate for PlaylistWithEntries {
    type Invalidity = PlaylistWithEntriesInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self { playlist, entries } = self;
        let context = ValidationContext::new().validate_with(playlist, Self::Invalidity::Playlist);
        entries
            .iter()
            .enumerate()
            .fold(context, |context, (index, entry)| {
                context.validate_with(entry, |invalidity| {
                    Self::Invalidity::Entry(index, invalidity)
                })
            })
            .into()
    }
}

pub type EntityWithEntries =
    crate::entity::Entity<EntityType, PlaylistWithEntries, PlaylistWithEntriesInvalidity>;

impl From<(Entity, Vec<Entry>)> for EntityWithEntries {
    fn from(from: (Entity, Vec<Entry>)) -> Self {
        let (entity, entries) = from;
        let (hdr, body) = entity.into();
        Self::new(
            EntityHeaderTyped::from_untyped(hdr.into_untyped()),
            PlaylistWithEntries {
                playlist: body,
                entries,
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntriesSummary {
    pub total_count: usize,

    pub added_ts_minmax: Option<(Timestamp, Timestamp)>,

    pub tracks: TracksSummary,
}

impl EntriesSummary {
    pub const EMPTY: Self = Self {
        total_count: 0,
        added_ts_minmax: None,
        tracks: TracksSummary::EMPTY,
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TracksSummary {
    pub total_count: usize,
    pub distinct_count: usize,
}

impl TracksSummary {
    pub const EMPTY: Self = Self {
        total_count: 0,
        distinct_count: 0,
    };
}

impl PlaylistWithEntries {
    #[must_use]
    pub fn entries_summary(&self) -> EntriesSummary {
        EntriesSummary {
            total_count: self.entries.len(),
            added_ts_minmax: self.entries_added_ts_minmax(),
            tracks: TracksSummary {
                total_count: self.count_tracks(),
                distinct_count: self.count_distinct_tracks(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistWithEntriesSummary {
    pub playlist: Playlist,

    pub entries: EntriesSummary,
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
