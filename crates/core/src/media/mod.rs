// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use mime::Mime;
use semval::prelude::*;

use crate::util::clock::OffsetDateTimeMs;

pub mod artwork;
use self::artwork::{Artwork, ArtworkInvalidity};

pub mod content;
use self::content::{
    AudioContentMetadataInvalidity, ContentLink, ContentMetadata, ContentMetadataFlags,
    ContentMetadataFlagsInvalidity,
};

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
        Cow::Owned(format!("{encoded_by} {encoder_settings}"))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Content {
    /// Revisioned link to an external source with the actual contents
    pub link: ContentLink,

    pub r#type: Mime,

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
    pub digest: Option<Vec<u8>>,

    pub metadata: ContentMetadata,

    pub metadata_flags: ContentMetadataFlags,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub collected_at: OffsetDateTimeMs,

    pub content: Content,

    pub artwork: Option<Artwork>,
}

#[derive(Copy, Clone, Debug)]
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
            content:
                Content {
                    link:
                        ContentLink {
                            path: link_path,
                            rev: _,
                        },
                    r#type: content_type,
                    metadata: content_metadata,
                    metadata_flags: content_metadata_flags,
                    digest: _,
                },
            artwork,
        } = self;
        let context = ValidationContext::new()
            .invalidate_if(
                link_path.as_str().trim().is_empty(),
                Self::Invalidity::LinkPathEmpty,
            )
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
