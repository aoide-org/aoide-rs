// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, fmt};

use bitflags::bitflags;
use strum::FromRepr;

use crate::{
    audio::{
        channel::{Channels, ChannelsInvalidity},
        signal::{
            BitrateBps, BitrateBpsInvalidity, LoudnessLufs, LoudnessLufsInvalidity, SampleRateHz,
            SampleRateHzInvalidity,
        },
        DurationMs, DurationMsInvalidity,
    },
    prelude::{url::BaseUrl, *},
};

pub mod resolver;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentPath<'a>(Cow<'a, str>);

impl<'a> ContentPath<'a> {
    #[must_use]
    pub const fn new(inner: Cow<'a, str>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn into_inner(self) -> Cow<'a, str> {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub fn to_borrowed(&'a self) -> Self {
        Self::new(Cow::Borrowed(&self.0))
    }

    #[must_use]
    pub fn into_owned(self) -> ContentPath<'static> {
        ContentPath::new(Cow::Owned(self.0.into_owned()))
    }

    #[must_use]
    pub fn clone_owned(&self) -> ContentPath<'static> {
        self.to_borrowed().into_owned()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn len(&'a self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn as_str(&'a self) -> &'a str {
        self.0.as_ref()
    }

    #[must_use]
    pub fn is_terminal(&self) -> bool {
        !(self.is_empty() || self.0.ends_with('/'))
    }
}

impl<'a> From<Cow<'a, str>> for ContentPath<'a> {
    fn from(from: Cow<'a, str>) -> Self {
        Self::new(from)
    }
}

impl<'a> From<&'a str> for ContentPath<'a> {
    fn from(from: &'a str) -> Self {
        Self::new(Cow::Borrowed(from))
    }
}

impl From<String> for ContentPath<'static> {
    fn from(from: String) -> Self {
        Self::new(Cow::Owned(from))
    }
}

impl From<ContentPath<'static>> for String {
    fn from(from: ContentPath<'static>) -> Self {
        from.into_inner().into_owned()
    }
}

impl<'a> AsRef<Cow<'a, str>> for ContentPath<'a> {
    fn as_ref(&self) -> &Cow<'a, str> {
        &self.0
    }
}

impl fmt::Display for ContentPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromRepr)]
#[repr(u8)]
pub enum ContentPathKind {
    /// Percent-encoded, canonical URI (case-sensitive)
    Uri = 0,

    /// Percent-encoded, canonical URL (case-sensitive)
    Url = 1,

    /// Percent-encoded, canonical URL with the scheme "file" (case-sensitive)
    FileUrl = 2,

    /// Relative file path with '/' as path separator (case-sensitive)
    ///
    /// An accompanying root or base URL must be provided by the outer context
    /// to reconstruct the corresponding `file://` URL.
    ///
    /// Relative file paths are NOT percent-encoded, i.e. may contain reserved
    /// characters like ' ', '#', or '?'.
    VirtualFilePath = 3,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentPathConfig {
    Uri,
    Url,
    FileUrl,
    VirtualFilePath { root_url: BaseUrl },
}

impl ContentPathConfig {
    #[must_use]
    pub const fn kind(&self) -> ContentPathKind {
        match self {
            Self::Uri => ContentPathKind::Uri,
            Self::Url => ContentPathKind::Url,
            Self::FileUrl => ContentPathKind::FileUrl,
            Self::VirtualFilePath { .. } => ContentPathKind::VirtualFilePath,
        }
    }

    #[must_use]
    pub const fn root_url(&self) -> Option<&BaseUrl> {
        match self {
            Self::VirtualFilePath { root_url } => Some(root_url),
            Self::Uri | Self::Url | Self::FileUrl => None,
        }
    }
}

/// Composition
impl TryFrom<(ContentPathKind, Option<BaseUrl>)> for ContentPathConfig {
    type Error = anyhow::Error;

    fn try_from((path_kind, root_url): (ContentPathKind, Option<BaseUrl>)) -> anyhow::Result<Self> {
        use ContentPathKind::*;
        let into = match path_kind {
            Uri => Self::Uri,
            Url => Self::Url,
            FileUrl => Self::FileUrl,
            VirtualFilePath => {
                if let Some(root_url) = root_url {
                    Self::VirtualFilePath { root_url }
                } else {
                    anyhow::bail!("Missing root URL");
                }
            }
        };
        Ok(into)
    }
}

/// Decomposition
impl From<ContentPathConfig> for (ContentPathKind, Option<BaseUrl>) {
    fn from(from: ContentPathConfig) -> Self {
        use ContentPathConfig::*;
        match from {
            Uri => (ContentPathKind::Uri, None),
            Url => (ContentPathKind::Url, None),
            FileUrl => (ContentPathKind::FileUrl, None),
            VirtualFilePath { root_url } => (ContentPathKind::VirtualFilePath, Some(root_url)),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ContentPathConfigInvalidity {
    RootUrl,
}

impl Validate for ContentPathConfig {
    type Invalidity = ContentPathConfigInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new();
        if let Self::VirtualFilePath { root_url } = self {
            context = context.invalidate_if(!root_url.is_file(), Self::Invalidity::RootUrl);
        }
        context.into()
    }
}

pub type ContentRevisionValue = u64;

pub type ContentRevisionSignedValue = i64;

/// Revision number representing last, synchronized state of an associated
/// external source
///
/// The external revision number is supposed to be strongly monotonic, i.e.
/// is increased by an arbitrary amount > 0 if the external source has been
/// modified. It is supposed to be updated after the internal contents have
/// been synchronized with the external source, i.e. both when importing or
/// exporting metadata.
///
/// Example: For local files the duration in milliseconds since Unix
/// epoch origin at 1970-01-01T00:00:00Z of the last modification time
/// provided by the file system is stored as the external revision number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentRevision(ContentRevisionValue);

impl ContentRevision {
    #[must_use]
    pub const fn new(val: ContentRevisionValue) -> Self {
        Self(val)
    }

    #[must_use]
    pub fn from_value(from: impl Into<ContentRevisionValue>) -> Self {
        Self::new(from.into())
    }

    #[must_use]
    pub const fn to_value(self) -> ContentRevisionValue {
        let Self(val) = self;
        val
    }

    #[must_use]
    pub fn from_signed_value(val: ContentRevisionSignedValue) -> Self {
        debug_assert!(val >= 0);
        Self::new(val as ContentRevisionValue)
    }

    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn to_signed_value(self) -> ContentRevisionSignedValue {
        debug_assert!(self <= Self::from_signed_value(ContentRevisionSignedValue::MAX));
        self.to_value() as ContentRevisionSignedValue
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn try_from_file_time(
        file_time: std::time::SystemTime,
    ) -> Result<Option<Self>, std::time::SystemTimeError> {
        if file_time == std::time::SystemTime::UNIX_EPOCH {
            // Only consider time stamps strictly after the epoch origin
            // meaningful and valid, e.g. in NixOS files may not have a
            // meaningful time stamp.
            return Ok(None);
        }
        file_time
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|duration| {
                let timestamp_millis = duration.as_millis();
                debug_assert!(timestamp_millis <= ContentRevisionValue::MAX.into());
                Some(Self::new(timestamp_millis as ContentRevisionValue))
            })
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn try_from_file(file: &std::fs::File) -> std::io::Result<Option<Self>> {
        let file_last_modified = file.metadata()?.modified()?;
        Self::try_from_file_time(file_last_modified)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }
}

impl From<ContentRevisionValue> for ContentRevision {
    fn from(from: ContentRevisionValue) -> Self {
        Self::new(from)
    }
}

impl From<ContentRevision> for ContentRevisionValue {
    fn from(from: ContentRevision) -> Self {
        from.to_value()
    }
}

impl fmt::Display for ContentRevision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.to_value()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentLink {
    pub path: ContentPath<'static>,
    pub rev: Option<ContentRevision>,
}

bitflags! {
    /// A bitmask for controlling how and if content metadata is
    /// re-imported from the source.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    #[must_use]
    pub const fn is_valid(self) -> bool {
        Self::all().contains(self)
    }

    #[must_use]
    pub fn is_unreliable(self) -> bool {
        !self.intersects(Self::RELIABLE | Self::LOCKED)
    }

    #[must_use]
    pub const fn is_reliable(self) -> bool {
        self.intersects(Self::RELIABLE)
    }

    #[must_use]
    pub const fn is_locked(self) -> bool {
        self.intersects(Self::LOCKED)
    }

    #[must_use]
    pub const fn is_stale(self) -> bool {
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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
pub enum ContentMetadata {
    Audio(AudioContentMetadata),
}

impl From<AudioContentMetadata> for ContentMetadata {
    fn from(from: AudioContentMetadata) -> Self {
        Self::Audio(from)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AudioContentMetadata {
    pub duration: Option<DurationMs>,

    pub channels: Option<Channels>,

    pub sample_rate: Option<SampleRateHz>,

    pub bitrate: Option<BitrateBps>,

    pub loudness: Option<LoudnessLufs>,

    // Encoder and settings
    pub encoder: Option<String>,
}

#[derive(Copy, Clone, Debug)]
pub enum AudioContentMetadataInvalidity {
    Duration(DurationMsInvalidity),
    Channels(ChannelsInvalidity),
    SampleRate(SampleRateHzInvalidity),
    Bitrate(BitrateBpsInvalidity),
    Loudness(LoudnessLufsInvalidity),
    EncoderEmpty,
}

impl Validate for AudioContentMetadata {
    type Invalidity = AudioContentMetadataInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.duration, Self::Invalidity::Duration)
            .validate_with(&self.channels, Self::Invalidity::Channels)
            .validate_with(&self.sample_rate, Self::Invalidity::SampleRate)
            .validate_with(&self.bitrate, Self::Invalidity::Bitrate)
            .validate_with(&self.loudness, Self::Invalidity::Loudness)
            .invalidate_if(
                self.encoder
                    .as_deref()
                    .map(str::trim)
                    .map_or(false, str::is_empty),
                Self::Invalidity::EncoderEmpty,
            )
            .into()
    }
}
