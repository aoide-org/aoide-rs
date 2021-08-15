// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::{
    audio::{AudioContent, AudioContentInvalidity},
    prelude::*,
};

use bitflags::bitflags;
use num_derive::{FromPrimitive, ToPrimitive};
use std::{
    borrow::Cow,
    fmt,
    ops::{Deref, DerefMut},
};

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct SourcePath(String);

impl SourcePath {
    pub const fn new(inner: String) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> String {
        let Self(inner) = self;
        inner
    }
}

impl From<String> for SourcePath {
    fn from(from: String) -> Self {
        Self::new(from)
    }
}

impl From<SourcePath> for String {
    fn from(from: SourcePath) -> Self {
        from.into_inner()
    }
}

impl AsRef<str> for &SourcePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for SourcePath {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

impl DerefMut for SourcePath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for SourcePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum SourcePathKind {
    /// Percent-encoded URI (case-sensitive)
    Uri = 0,

    /// Percent-encoded URL (case-sensitive)
    Url = 1,

    /// Percent-encoded URL with the scheme "file" (case-sensitive).
    FileUrl = 2,

    /// Case-sensitive, portable file path with '/' as path separator.
    ///
    /// Either absolute or relative to some base "file" URL.
    VirtualFilePath = 3,
}

///////////////////////////////////////////////////////////////////////
// Content
///////////////////////////////////////////////////////////////////////

bitflags! {
    /// A bitmask for controlling how and if content metadata is
    /// re-imported from the source.
    pub struct ContentMetadataFlags: u8 {
        /// Use case: Parsed from file tags which are considered inaccurate
        /// and are often imprecise.
        const UNRELIABLE = 0b0000_0000;

        /// Use case: Reported by a decoder when opening an audio/video
        /// stream for reading. Nevertheless different decoders may report
        /// slightly differing values.
        const RELIABLE   = 0b0000_0001;

        /// Locked metadata will not be updated automatically, neither when
        /// parsing file tags nor when decoding an audio/video stream.
        ///
        /// While locked the stale flag is never set.
        const LOCKED     = 0b0000_0010;

        /// Stale metadata should be re-imported depending on the other
        /// flags.
        const STALE      = 0b0000_0100;
    }
}

impl ContentMetadataFlags {
    pub fn is_valid(self) -> bool {
        Self::all().contains(self)
    }

    pub fn is_unreliable(self) -> bool {
        !self.intersects(Self::RELIABLE | Self::LOCKED)
    }

    pub fn is_reliable(self) -> bool {
        self.intersects(Self::RELIABLE)
    }

    pub fn is_locked(self) -> bool {
        self.intersects(Self::LOCKED)
    }

    pub fn is_stale(self) -> bool {
        self.intersects(Self::STALE)
    }

    /// Update the current state
    ///
    /// If the given target state is considered at least as reliable
    /// as the current state then modifications are allowed by returning
    /// `true` and the new target state is established.
    ///
    /// Otherwise the current state is preserved. The return value
    /// `false` indicates that modification of metadata is not desired
    /// to prevent loss of accuracy or precision. Instead the stale flag
    /// is set (only if currently not locked) to indicate that an update
    /// from a more reliable source of metadata should be considered.
    ///
    /// The given target state MUST NOT be marked as stale!
    pub fn update(&mut self, target: Self) -> bool {
        debug_assert!(!target.is_stale());
        if (*self - Self::STALE) == target
            || target.is_locked()
            || (!self.is_locked() && target.is_reliable())
        {
            *self = target;
            true
        } else {
            // Metadata does not get stale while locked
            if !self.is_locked() {
                *self |= Self::STALE;
            }
            false
        }
    }
}

impl Default for ContentMetadataFlags {
    fn default() -> Self {
        Self::UNRELIABLE
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct ContentMetadataFlagsInvalidity;

impl Validate for ContentMetadataFlags {
    type Invalidity = ContentMetadataFlagsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !ContentMetadataFlags::is_valid(*self),
                ContentMetadataFlagsInvalidity,
            )
            .into()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    Audio(AudioContent),
}

impl From<AudioContent> for Content {
    fn from(from: AudioContent) -> Self {
        Self::Audio(from)
    }
}

///////////////////////////////////////////////////////////////////////
// Encoder
///////////////////////////////////////////////////////////////////////

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

///////////////////////////////////////////////////////////////////////
// Artwork
///////////////////////////////////////////////////////////////////////

/// The APIC picture type code as defined by ID3v2.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum ApicType {
    Other = 0x00,
    Icon = 0x01,
    OtherIcon = 0x02,
    CoverFront = 0x03,
    CoverBack = 0x04,
    Leaflet = 0x05,
    Media = 0x06,
    LeadArtist = 0x07,
    Artist = 0x08,
    Conductor = 0x09,
    Band = 0x0A,
    Composer = 0x0B,
    Lyricist = 0x0C,
    RecordingLocation = 0x0D,
    DuringRecording = 0x0E,
    DuringPerformance = 0x0F,
    ScreenCapture = 0x10,
    BrightFish = 0x11,
    Illustration = 0x12,
    BandLogo = 0x13,
    PublisherLogo = 0x14,
}

impl ApicType {
    pub fn try_from_u8(val: u8) -> Option<Self> {
        let some = match val {
            0x00 => Self::Other,
            0x01 => Self::Icon,
            0x02 => Self::OtherIcon,
            0x03 => Self::CoverFront,
            0x04 => Self::CoverBack,
            0x05 => Self::Leaflet,
            0x06 => Self::Media,
            0x07 => Self::LeadArtist,
            0x08 => Self::Artist,
            0x09 => Self::Conductor,
            0x0A => Self::Band,
            0x0B => Self::Composer,
            0x0C => Self::Lyricist,
            0x0D => Self::RecordingLocation,
            0x0E => Self::DuringRecording,
            0x0F => Self::DuringPerformance,
            0x10 => Self::ScreenCapture,
            0x11 => Self::BrightFish,
            0x12 => Self::Illustration,
            0x13 => Self::BandLogo,
            0x14 => Self::PublisherLogo,
            _ => return None,
        };
        Some(some)
    }

    pub const fn to_u8(self) -> u8 {
        self as u8
    }
}

pub type ImageDimension = u16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ImageSize {
    pub width: ImageDimension,
    pub height: ImageDimension,
}

impl ImageSize {
    pub const fn is_empty(self) -> bool {
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

pub type Digest = [u8; 32];

pub type Thumbnail4x4Rgb8 = [u8; 4 * 4 * 3];

/// Artwork image properties
///
/// All properties are optional for maximum flexibility.
/// Properties could be missing or are yet unknown at some point
/// in time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtworkImage {
    /// The media type, e.g. "image/jpeg"
    pub media_type: String,

    pub apic_type: ApicType,

    /// The dimensions of the image (if known).
    pub size: Option<ImageSize>,

    /// Identifies the actual content, e.g. for cache lookup or to detect
    /// modifications.
    pub digest: Option<Digest>,

    /// A 4x4 R8G8B8 thumbnail image.
    pub thumbnail: Option<Thumbnail4x4Rgb8>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ArtworkImageInvalidity {
    MediaTypeEmpty,
    Size(ImageSizeInvalidity),
}

impl Validate for ArtworkImage {
    type Invalidity = ArtworkImageInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.media_type.is_empty(), Self::Invalidity::MediaTypeEmpty)
            .validate_with(&self.size, Self::Invalidity::Size)
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedArtwork {
    pub image: ArtworkImage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedArtwork {
    /// Absolute or relative URI/URL that links to the image.
    pub uri: String,

    pub image: ArtworkImage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Artwork {
    /// Artwork has been looked up at least once but nothing has been found.
    Missing,

    /// The artwork is embedded in the media source.
    Embedded(EmbeddedArtwork),

    /// The artwork references an external image.
    Linked(LinkedArtwork),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ArtworkInvalidity {
    Image(ArtworkImageInvalidity),
}

impl Validate for Artwork {
    type Invalidity = ArtworkInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new();
        match self {
            Self::Missing => (),
            Self::Embedded(embedded) => {
                context = context.validate_with(&embedded.image, Self::Invalidity::Image);
            }
            Self::Linked(linked) => {
                // TODO: Validate uri
                context = context.validate_with(&linked.image, Self::Invalidity::Image);
            }
        }
        context.into()
    }
}

///////////////////////////////////////////////////////////////////////
// Source
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub collected_at: DateTime,

    pub synchronized_at: Option<DateTime>,

    pub path: SourcePath,

    pub content_type: String,

    /// Content digest for identifying sources independent of their
    /// URI, e.g. to detect moved files.
    ///
    /// The digest should be calculated from the raw stream data
    /// that is supposed to be read-only and immutable over time.
    /// Additional metadata like file tags that is modified
    /// frequently is not suitable to be included in the digest
    /// calculation.
    pub content_digest: Option<Vec<u8>>,

    pub content_metadata_flags: ContentMetadataFlags,

    pub content: Content,

    pub artwork: Option<Artwork>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SourceInvalidity {
    PathEmpty,
    ContentTypeEmpty,
    ContentMetadataFlags(ContentMetadataFlagsInvalidity),
    AudioContent(AudioContentInvalidity),
    Artwork(ArtworkInvalidity),
}

impl Validate for Source {
    type Invalidity = SourceInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let Self {
            artwork,
            collected_at: _,
            content,
            content_digest: _,
            content_metadata_flags,
            content_type,
            path,
            synchronized_at: _,
        } = self;
        let context = ValidationContext::new()
            .invalidate_if(path.trim().is_empty(), Self::Invalidity::PathEmpty)
            .invalidate_if(
                content_type.trim().is_empty(),
                Self::Invalidity::ContentTypeEmpty,
            )
            .validate_with(
                &content_metadata_flags,
                Self::Invalidity::ContentMetadataFlags,
            )
            .validate_with(&artwork, Self::Invalidity::Artwork);
        // TODO: Validate MIME type
        match content {
            Content::Audio(ref audio_content) => {
                context.validate_with(audio_content, Self::Invalidity::AudioContent)
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
