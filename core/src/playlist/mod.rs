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

///////////////////////////////////////////////////////////////////////

use super::*;

use crate::{
    entity::{EntityUid, EntityUidInvalidity},
    util::{clock::TickInstant, color::Color},
};

use rand::{seq::SliceRandom, thread_rng};
use std::ops::RangeBounds;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistTrack {
    /// A reference to the track.
    pub uid: EntityUid,
}

#[derive(Copy, Clone, Debug)]
pub enum PlaylistTrackInvalidity {
    Uid(EntityUidInvalidity),
}

impl Validate for PlaylistTrack {
    type Invalidity = PlaylistTrackInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.uid, Self::Invalidity::Uid)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaylistItem {
    Separator, // empty
    Track(PlaylistTrack),
    // TODO: Add more items like an optional transition between
    // two subsequent track items?
    //Transition(PlaylistTransition),
}

impl PlaylistItem {
    pub fn is_track(&self) -> bool {
        if let Self::Track(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PlaylistItemInvalidity {
    Track(PlaylistTrackInvalidity),
}

impl Validate for PlaylistItem {
    type Invalidity = PlaylistItemInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        use PlaylistItem::*;
        match self {
            Separator => context,
            Track(ref track) => context.validate_with(track, Self::Invalidity::Track),
        }
        .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistEntry {
    /// The actual item, e.g. a single track or a transition between
    /// two subsequent tracks.
    pub item: PlaylistItem,

    /// Time stamp added when this entry is part of the playlist,
    /// i.e. when it has been created and added.
    pub added: TickInstant,
}

#[derive(Copy, Clone, Debug)]
pub enum PlaylistEntryInvalidity {
    Item(PlaylistItemInvalidity),
}

impl Validate for PlaylistEntry {
    type Invalidity = PlaylistEntryInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.item, Self::Invalidity::Item)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Playlist {
    /// Mandatory name.
    pub name: String,

    /// Optional description.
    pub description: Option<String>,

    /// Custom type of the playlist.
    ///
    /// This property allows 3rd party applications to distinguish
    /// different kinds of playlists for different purposes and depending
    /// on their use case, e.g. generated session or history playlists for
    /// logging all tracks that have been played during this session.
    pub r#type: Option<String>,

    /// Optional color for display purposes.
    pub color: Option<Color>,

    /// Ordered list of playlist entries.
    pub entries: Vec<PlaylistEntry>,
}

impl Playlist {
    pub fn entries_added_minmax(&self) -> Option<(TickInstant, TickInstant)> {
        let mut entries = self.entries.iter();
        if let Some(first_added) = entries.next().map(|e| e.added) {
            let mut added_min = first_added;
            let mut added_max = first_added;
            for e in entries {
                added_min = added_min.min(e.added);
                added_max = added_max.max(e.added);
            }
            Some((added_min, added_max))
        } else {
            None
        }
    }

    pub fn append_entries(&mut self, new_entries: impl IntoIterator<Item = PlaylistEntry>) {
        self.replace_entries(self.entries.len().., new_entries);
    }

    pub fn insert_entries(
        &mut self,
        before: usize,
        new_entries: impl IntoIterator<Item = PlaylistEntry>,
    ) {
        self.replace_entries(before..before, new_entries);
    }

    pub fn replace_entries(
        &mut self,
        range: impl RangeBounds<usize>,
        new_entries: impl IntoIterator<Item = PlaylistEntry>,
    ) {
        self.entries.splice(range, new_entries.into_iter());
    }

    pub fn remove_entries(&mut self, range: impl RangeBounds<usize>) {
        self.entries.drain(range);
    }

    pub fn remove_all_entries(&mut self) {
        self.entries.clear();
    }

    pub fn shuffle_entries(&mut self) {
        self.entries.shuffle(&mut thread_rng());
    }

    // Sort entries by their creation time stamp, preserving the
    // order of entries with equal time stamps.
    pub fn reverse_entries(&mut self) {
        self.entries.reverse();
    }

    // Sort entries by their creation time stamp, preserving the
    // order of entries with equal time stamps.
    pub fn sort_entries_chronologically(&mut self) {
        self.entries.sort_by_key(|e| e.added);
    }

    pub fn count_tracks(&self) -> usize {
        self.entries.iter().filter(|e| e.item.is_track()).count()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PlaylistInvalidity {
    Name,
    Entry(usize, PlaylistEntryInvalidity),
}

impl Validate for Playlist {
    type Invalidity = PlaylistInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context =
            ValidationContext::new().invalidate_if(self.name.is_empty(), PlaylistInvalidity::Name);
        self.entries
            .iter()
            .enumerate()
            .fold(context, |context, (index, entry)| {
                context.validate_with(entry, |invalidity| {
                    PlaylistInvalidity::Entry(index, invalidity)
                })
            })
            .into()
    }
}

pub type Entity = crate::entity::Entity<PlaylistInvalidity, Playlist>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistBriefEntries {
    pub count: usize,

    pub added_minmax: Option<(TickInstant, TickInstant)>,

    pub tracks: PlaylistBriefTracks,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistBriefTracks {
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaylistBrief {
    pub name: String,

    pub description: Option<String>,

    pub r#type: Option<String>,

    pub color: Option<Color>,

    pub entries: PlaylistBriefEntries,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaylistBriefRef<'a> {
    pub name: &'a str,

    pub description: Option<&'a str>,

    pub r#type: Option<&'a str>,

    pub color: Option<Color>,

    pub entries: PlaylistBriefEntries,
}

impl<'a> Playlist {
    pub fn entries_brief(&self) -> PlaylistBriefEntries {
        let tracks = PlaylistBriefTracks {
            count: self.count_tracks(),
        };
        PlaylistBriefEntries {
            count: self.entries.len(),
            added_minmax: self.entries_added_minmax(),
            tracks,
        }
    }

    pub fn brief_ref(&'a self) -> PlaylistBriefRef<'a> {
        let entries = self.entries_brief();
        let Playlist {
            ref name,
            ref description,
            r#type,
            color,
            entries: _entries,
        } = self;
        PlaylistBriefRef {
            name,
            description: description.as_deref(),
            r#type: r#type.as_deref(),
            color: *color,
            entries,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
