// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::borrow::Cow;

use mime::Mime;
use num_derive::{FromPrimitive, ToPrimitive};

use crate::prelude::*;

use self::{
    artwork::{Artwork, ArtworkInvalidity},
    content::{
        AudioContentMetadataInvalidity, ContentLink, ContentMetadata, ContentMetadataFlags,
        ContentMetadataFlagsInvalidity,
    },
};

pub mod artwork;
pub mod content;

/// Concatenate encoder properties
///
/// Some but not all file formats specify two different encoder
/// properties, namely *encoded by* and *encoder settings*. In
/// aoide those properties are represented by a single string.
///
/// Either of the strings might be empty if unknown.
pub fn concat_encoder_strings<'a, T>(encoded_by: &'a T, encoder_settings: &'a T) -> Cow<'a, str>
where
    T: AsRef<str> + ?Sized,
{
    let encoded_by = encoded_by.as_ref().trim();
    let encoder_settings = encoder_settings.as_ref().trim();
    if encoded_by.is_empty() {
        Cow::Borrowed(encoder_settings)
    } else if encoder_settings.is_empty() {
        Cow::Borrowed(encoded_by)
    } else {
        // Concatenate both strings into a single field
        debug_assert!(!encoded_by.is_empty());
        debug_assert!(!encoder_settings.is_empty());
        Cow::Owned(format!("{} {}", encoded_by, encoder_settings))
    }
}

/// Concatenate encoder properties
///
/// Some but not all file formats specify two different encoder
/// properties, namely *encoded by* and *encoder settings*. In
/// aoide those properties are represented by a single string.
///
/// Both properties are optional.
#[must_use]
pub fn concat_encoder_properties<'a>(
    encoded_by: Option<&'a str>,
    encoder_settings: Option<&'a str>,
) -> Option<Cow<'a, str>> {
    let encoder = concat_encoder_strings(
        encoded_by.unwrap_or_default(),
        encoder_settings.unwrap_or_default(),
    );
    if encoder.is_empty() {
        None
    } else {
        Some(encoder)
    }
}

/// Advisory rating code for content(s)
///
/// Values match the "rtng" MP4 atom containing the advisory rating
/// as written by iTunes.
///
/// Note: Previously Apple used the value 4 for explicit content that
/// has now been replaced by 1.
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum AdvisoryRating {
    /// Inoffensive
    Unrated = 0,

    /// Offensive
    Explicit = 1,

    /// Inoffensive (Edited)
    Clean = 2,
}

impl AdvisoryRating {
    #[must_use]
    pub fn is_offensive(self) -> bool {
        match self {
            Self::Unrated | Self::Clean => false,
            Self::Explicit => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub collected_at: DateTime,

    /// Revisioned link to an external source with the actual contents
    pub content_link: ContentLink,

    pub content_type: Mime,

    /// Digest of immutable content data
    ///
    /// Fingerprint for identifying sources independent of their URI,
    /// e.g. to detect moved files.
    ///
    /// The fingerprint should be calculated from the raw stream data
    /// that is supposed to be read-only and immutable over time.
    /// Additional metadata like file tags that are modified frequently
    /// and change over time are not suitable to be included in the
    /// calculation of the fingerprint.
    pub content_digest: Option<Vec<u8>>,

    pub content_metadata: ContentMetadata,

    pub content_metadata_flags: ContentMetadataFlags,

    pub artwork: Option<Artwork>,

    pub advisory_rating: Option<AdvisoryRating>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SourceInvalidity {
    LinkPathEmpty,
    ContentTypeEmpty,
    ContentMetadataFlags(ContentMetadataFlagsInvalidity),
    AudioContentMetadata(AudioContentMetadataInvalidity),
    Artwork(ArtworkInvalidity),
}

impl Validate for Source {
    type Invalidity = SourceInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self {
            collected_at: _,
            content_link:
                ContentLink {
                    path: link_path,
                    rev: _,
                },
            content_type,
            content_metadata,
            content_metadata_flags,
            content_digest: _,
            advisory_rating: _,
            artwork,
        } = self;
        let context = ValidationContext::new()
            .invalidate_if(link_path.trim().is_empty(), Self::Invalidity::LinkPathEmpty)
            .invalidate_if(
                content_type.essence_str().is_empty(),
                Self::Invalidity::ContentTypeEmpty,
            )
            .validate_with(
                &content_metadata_flags,
                Self::Invalidity::ContentMetadataFlags,
            )
            .validate_with(&artwork, Self::Invalidity::Artwork);
        // TODO: Validate MIME type
        match content_metadata {
            ContentMetadata::Audio(ref audio_content) => {
                context.validate_with(audio_content, Self::Invalidity::AudioContentMetadata)
            }
        }
        .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
