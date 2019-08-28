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

use crate::audio::{AudioContent, AudioContentInvalidity};

///////////////////////////////////////////////////////////////////////
// MediaContent
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub enum MediaContent {
    Audio(AudioContent),
}

///////////////////////////////////////////////////////////////////////
// MediaSource
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct MediaSource {
    pub uri: String,

    // The content_type uniquely identifies a MediaSource of
    // a Track, i.e. no duplicate content types are allowed
    // among the track sources of each track.
    pub content_type: String,

    pub content: MediaContent,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MediaSourceInvalidity {
    UriEmpty,
    ContentTypeEmpty,
    AudioContent(AudioContentInvalidity),
}

impl Validate for MediaSource {
    type Invalidity = MediaSourceInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new()
            .invalidate_if(self.uri.trim().is_empty(), MediaSourceInvalidity::UriEmpty)
            .invalidate_if(
                self.content_type.trim().is_empty(),
                MediaSourceInvalidity::ContentTypeEmpty,
            );
        // TODO: Validate MIME type
        match self.content {
            MediaContent::Audio(ref audio_content) => {
                context.validate_and_map(audio_content, MediaSourceInvalidity::AudioContent)
            }
        }
        .into()
    }
}

#[derive(Debug)]
pub struct MediaSources;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MediaSourcesInvalidity {
    MediaSource(MediaSourceInvalidity),
    TypeAmbiguous,
}

impl MediaSources {
    pub fn validate<'a, I>(sources: I) -> ValidationResult<MediaSourcesInvalidity>
    where
        I: Iterator<Item = &'a MediaSource> + Clone,
    {
        let mut context = sources
            .clone()
            .fold(ValidationContext::new(), |context, source| {
                context.validate_and_map(source, MediaSourcesInvalidity::MediaSource)
            });
        if context.is_valid() {
            let mut content_types: Vec<_> =
                sources.clone().map(|source| &source.content_type).collect();
            content_types.sort_unstable();
            content_types.dedup();
            context = context.invalidate_if(
                content_types.len() < sources.count(),
                MediaSourcesInvalidity::TypeAmbiguous,
            );
        }
        context.into()
    }

    pub fn filter_content_type<'a, 'b, I>(
        sources: I,
        content_type: &'b str,
    ) -> impl Iterator<Item = &'a MediaSource>
    where
        I: Iterator<Item = &'a MediaSource>,
        'b: 'a,
    {
        sources.filter(move |source| source.content_type == content_type)
    }
}
