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
    util::{clock::TickInstant, color::ColorRgb},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistEntry {
    /// A reference to the track.
    pub track_uid: EntityUid,

    /// Time stamp since when this entry is part of its playlist,
    /// i.e. when it has been created and added.
    pub since: TickInstant,

    /// Custom comments and notes to annotate this entry.
    pub comment: Option<String>,
}

#[derive(Copy, Clone, Debug)]
pub enum PlaylistEntryInvalidity {
    TrackUid(EntityUidInvalidity),
}

impl Validate for PlaylistEntry {
    type Invalidity = PlaylistEntryInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.track_uid, PlaylistEntryInvalidity::TrackUid)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub color: Option<ColorRgb>,

    /// Ordered list of playlist entries.
    pub entries: Vec<PlaylistEntry>,
}

impl Playlist {
    pub fn entries_since_min_max(&self) -> Option<(TickInstant, TickInstant)> {
        let mut entries = self.entries.iter();
        if let Some(first_since) = entries.next().map(|e| e.since) {
            let mut since_min = first_since;
            let mut since_max = first_since;
            for e in entries {
                since_min = since_min.min(e.since);
                since_max = since_max.max(e.since);
            }
            Some((since_min, since_max))
        } else {
            None
        }
    }

    // Sort entries by their creation time stamp, preserving the
    // order of entries with equal time stamps.
    pub fn sort_entries_chronologically(&mut self) {
        self.entries.sort_by_key(|e| e.since);
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
pub struct PlaylistBrief<'a> {
    pub name: &'a str,

    pub description: Option<&'a str>,

    pub r#type: Option<&'a str>,

    pub color: Option<ColorRgb>,

    pub entries_count: usize,

    pub entries_since_min: Option<TickInstant>,

    pub entries_since_max: Option<TickInstant>,
}

impl<'a> Playlist {
    pub fn brief(&'a self) -> PlaylistBrief<'a> {
        let (entries_since_min, entries_since_max) = self
            .entries_since_min_max()
            .map_or((None, None), |(min, max)| (Some(min), Some(max)));
        let Playlist {
            ref name,
            ref description,
            r#type,
            color,
            ref entries,
        } = self;
        let entries_count = entries.len();
        PlaylistBrief {
            name,
            description: description.as_ref().map(String::as_str),
            r#type: r#type.as_ref().map(String::as_str),
            color: *color,
            entries_count,
            entries_since_min,
            entries_since_max,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
