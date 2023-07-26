// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::RangeBounds;

use bitflags::bitflags;
use rand::{seq::SliceRandom as _, RngCore};

use crate::{
    prelude::{random::adhoc_rng, *},
    TrackUid,
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
    pub fn is_separator(&self) -> bool {
        matches!(self, Self::Separator(_))
    }

    #[must_use]
    pub fn is_track(&self) -> bool {
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
            Item::Track(ref track) => context.validate_with(track, Self::Invalidity::Track),
        }
        .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Time stamp added when this entry is part of the playlist,
    /// i.e. when it has been created and added.
    pub added_at: OffsetDateTimeMs,

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
                title
                    .as_ref()
                    .map_or(false, |title| title.trim().is_empty()),
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
    pub fn is_valid(self) -> bool {
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
                kind.as_ref().map_or(false, |kind| kind.trim().is_empty()),
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
    pub fn entries_added_at_minmax(&self) -> Option<(OffsetDateTimeMs, OffsetDateTimeMs)> {
        let mut entries = self.entries.iter();
        if let Some(first_added) = entries.next().map(|e| e.added_at) {
            let mut added_min = first_added;
            let mut added_max = first_added;
            for e in entries {
                added_min = added_min.min(e.added_at);
                added_max = added_max.max(e.added_at);
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
        self.entries.sort_by_key(|e| e.added_at);
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

    pub added_at_minmax: Option<(OffsetDateTimeMs, OffsetDateTimeMs)>,

    pub tracks: TracksSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TracksSummary {
    pub total_count: usize,
    pub distinct_count: usize,
}

impl PlaylistWithEntries {
    #[must_use]
    pub fn entries_summary(&self) -> EntriesSummary {
        EntriesSummary {
            total_count: self.entries.len(),
            added_at_minmax: self.entries_added_at_minmax(),
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
