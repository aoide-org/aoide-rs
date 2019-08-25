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

use crate::audio::{AudioContent, AudioContentValidation};

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
pub enum MediaSourceValidation {
    UriEmpty,
    ContentTypeEmpty,
    AudioContent(AudioContentValidation),
}

impl Validate for MediaSource {
    type Validation = MediaSourceValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(self.uri.trim().is_empty(), MediaSourceValidation::UriEmpty);
        context.add_violation_if(
            self.content_type.trim().is_empty(),
            MediaSourceValidation::ContentTypeEmpty,
        );
        // TODO: Validate MIME type
        match self.content {
            MediaContent::Audio(ref audio_content) => context.map_and_merge_result(
                audio_content.validate(),
                MediaSourceValidation::AudioContent,
            ),
        }
        context.into_result()
    }
}

#[derive(Debug)]
pub struct MediaSources;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MediaSourcesValidation {
    MediaSource(MediaSourceValidation),
    TypeAmbiguous,
}

impl MediaSources {
    pub fn validate<'a, I>(sources: I) -> ValidationResult<MediaSourcesValidation>
    where
        I: Iterator<Item = &'a MediaSource> + Clone,
    {
        let mut context = ValidationContext::default();
        for source in sources.clone() {
            context.map_and_merge_result(source.validate(), MediaSourcesValidation::MediaSource);
        }
        if !context.has_violations() {
            let mut content_types: Vec<_> =
                sources.clone().map(|source| &source.content_type).collect();
            content_types.sort_unstable();
            content_types.dedup();
            context.add_violation_if(
                content_types.len() < sources.count(),
                MediaSourcesValidation::TypeAmbiguous,
            );
        }
        context.into_result()
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
