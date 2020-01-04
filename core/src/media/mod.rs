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

use super::*;

use crate::{
    audio::{AudioContent, AudioContentInvalidity},
    util::color::ColorRgb,
};

///////////////////////////////////////////////////////////////////////
// Content
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Audio(AudioContent),
}

///////////////////////////////////////////////////////////////////////
// Artwork
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ImageSize {
    pub width: u16,
    pub height: u16,
}

impl ImageSize {
    pub fn is_empty(self) -> bool {
        !(self.width > 0 && self.height > 0)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ImageSizeInvalidity {
    Empty,
}

impl Validate for ImageSize {
    type Invalidity = ImageSizeInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.is_empty(), ImageSizeInvalidity::Empty)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artwork {
    pub size: ImageSize,

    /// Identifies the actual content for cache lookup and to decide
    /// about modifications, e.g. a base64-encoded SHA256 hash of the
    /// image data.
    pub fingerprint: Option<String>,

    /// Selects one out of multiple resources embedded in the media source
    /// or an external resource.
    pub uri: Option<String>,

    /// The optional background color can be used to quickly display
    /// a preliminary view before the actual image has been loaded and
    /// for selecting a matching color scheme.
    pub background_color: Option<ColorRgb>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ArtworkInvalidity {
    ImageSize(ImageSizeInvalidity),
}

impl Validate for Artwork {
    type Invalidity = ArtworkInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.size, ArtworkInvalidity::ImageSize)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub uri: String,

    // The content_type uniquely identifies a Source of
    // a Track, i.e. no duplicate content types are allowed
    // among the track sources of each track.
    pub content_type: String,

    pub content: Content,

    pub artwork: Option<Artwork>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SourceInvalidity {
    UriEmpty,
    ContentTypeEmpty,
    AudioContent(AudioContentInvalidity),
    Artwork(ArtworkInvalidity),
}

impl Validate for Source {
    type Invalidity = SourceInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new()
            .invalidate_if(self.uri.trim().is_empty(), SourceInvalidity::UriEmpty)
            .invalidate_if(
                self.content_type.trim().is_empty(),
                SourceInvalidity::ContentTypeEmpty,
            )
            .validate_with(&self.artwork, SourceInvalidity::Artwork);
        // TODO: Validate MIME type
        match self.content {
            Content::Audio(ref audio_content) => {
                context.validate_with(audio_content, SourceInvalidity::AudioContent)
            }
        }
        .into()
    }
}

#[derive(Debug)]
pub struct Sources;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SourcesInvalidity {
    Source(SourceInvalidity),
    TypeAmbiguous,
}

impl Sources {
    pub fn validate<'a, I>(sources: I) -> ValidationResult<SourcesInvalidity>
    where
        I: Iterator<Item = &'a Source> + Clone,
    {
        let mut context = sources
            .clone()
            .fold(ValidationContext::new(), |context, source| {
                context.validate_with(source, SourcesInvalidity::Source)
            });
        if context.is_valid() {
            let mut content_types: Vec<_> =
                sources.clone().map(|source| &source.content_type).collect();
            content_types.sort_unstable();
            content_types.dedup();
            context = context.invalidate_if(
                content_types.len() < sources.count(),
                SourcesInvalidity::TypeAmbiguous,
            );
        }
        context.into()
    }

    pub fn filter_content_type<'a, 'b, I>(
        sources: I,
        content_type: &'b str,
    ) -> impl Iterator<Item = &'a Source>
    where
        I: Iterator<Item = &'a Source>,
        'b: 'a,
    {
        sources.filter(move |source| source.content_type == content_type)
    }
}
