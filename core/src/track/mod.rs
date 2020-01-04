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

pub mod album;
pub mod collection;
pub mod index;
pub mod marker;
pub mod release;
pub mod tag;

use self::{album::*, collection::*, index::*, marker::*, release::*};

use crate::{actor::*, media, tag::*, title::*};

///////////////////////////////////////////////////////////////////////
// TrackLock
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub collections: Vec<Collection>,

    pub media_sources: Vec<media::Source>,

    pub release: Option<Release>,

    pub album: Option<Album>,

    pub titles: Vec<Title>,

    pub actors: Vec<Actor>,

    pub indexes: Indexes,

    pub markers: Markers,

    pub tags: Vec<Tag>,
}

impl Track {
    pub fn purge_media_source_by_uri(&mut self, uri: &str) -> usize {
        let len_before = self.media_sources.len();
        self.media_sources
            .retain(|media_source| media_source.uri != uri);
        debug_assert!(self.media_sources.len() <= len_before);
        len_before - self.media_sources.len()
    }

    pub fn purge_media_source_by_uri_prefix(&mut self, uri_prefix: &str) -> usize {
        let len_before = self.media_sources.len();
        self.media_sources
            .retain(|media_source| !media_source.uri.starts_with(uri_prefix));
        debug_assert!(self.media_sources.len() <= len_before);
        len_before - self.media_sources.len()
    }

    pub fn relocate_media_source_by_uri(&mut self, old_uri: &str, new_uri: &str) -> usize {
        let mut relocated = 0;
        for mut media_source in &mut self.media_sources {
            if media_source.uri == old_uri {
                media_source.uri = new_uri.to_owned();
                relocated += 1;
            }
        }
        relocated
    }

    pub fn relocate_media_source_by_uri_prefix(
        &mut self,
        old_uri_prefix: &str,
        new_uri_prefix: &str,
    ) -> usize {
        let mut relocated = 0;
        for mut media_source in &mut self.media_sources {
            if media_source.uri.starts_with(old_uri_prefix) {
                let mut new_uri = String::with_capacity(
                    new_uri_prefix.len() + (media_source.uri.len() - old_uri_prefix.len()),
                );
                new_uri.push_str(new_uri_prefix);
                new_uri.push_str(&media_source.uri[old_uri_prefix.len()..]);
                media_source.uri = new_uri;
                relocated += 1;
            }
        }
        relocated
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TrackInvalidity {
    Collections(CollectionsInvalidity),
    MediaSources(media::SourcesInvalidity),
    Release(ReleaseInvalidity),
    Album(AlbumInvalidity),
    Titles(TitlesInvalidity),
    Actors(ActorsInvalidity),
    Indexes(IndexesInvalidity),
    Markers(MarkersInvalidity),
    Tags(TagsInvalidity),
}

impl Validate for Track {
    type Invalidity = TrackInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .merge_result_with(
                Collections::validate(self.collections.iter()),
                TrackInvalidity::Collections,
            )
            .merge_result_with(
                media::Sources::validate(self.media_sources.iter()),
                TrackInvalidity::MediaSources,
            )
            .merge_result_with(
                Titles::validate(self.titles.iter()),
                TrackInvalidity::Titles,
            )
            .merge_result_with(
                Actors::validate(self.actors.iter()),
                TrackInvalidity::Actors,
            )
            .validate_with(&self.album, TrackInvalidity::Album)
            .validate_with(&self.release, TrackInvalidity::Release)
            .validate_with(&self.indexes, TrackInvalidity::Indexes)
            .validate_with(&self.markers, TrackInvalidity::Markers)
            .merge_result_with(Tags::validate(self.tags.iter()), TrackInvalidity::Tags)
            .into()
    }
}

pub type Entity = crate::entity::Entity<TrackInvalidity, Track>;

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
