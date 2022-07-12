// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use lewton::{header::IdentHeader, inside_ogg::OggStreamReader, OggReadError, VorbisError};
use semval::IsValid as _;

use aoide_core::{
    audio::{
        channel::ChannelCount,
        signal::{BitrateBps, SampleRateHz},
    },
    media::{
        artwork::ApicType,
        content::{AudioContentMetadata, ContentMetadata, ContentMetadataFlags},
    },
    track::Track,
};

use crate::{io::import::*, Error, Result};

use super::vorbis;

fn map_vorbis_err(err: VorbisError) -> Error {
    match err {
        VorbisError::OggError(OggReadError::ReadError(err)) => Error::Io(err),
        err => Error::Other(anyhow::Error::from(err)),
    }
}

#[allow(missing_debug_implementations)]
pub struct Metadata(IdentHeader, Vec<(String, String)>);

impl Metadata {
    pub fn read_from(reader: &mut impl Reader) -> Result<Self> {
        OggStreamReader::new(reader)
            .map(|r| Self(r.ident_hdr, r.comment_hdr.comment_list))
            .map_err(map_vorbis_err)
    }

    #[must_use]
    pub fn find_embedded_artwork_image(
        &self,
        importer: &mut Importer,
    ) -> Option<(ApicType, String, Vec<u8>)> {
        vorbis::find_embedded_artwork_image(importer, &self.1)
    }

    #[must_use]
    pub fn import_audio_content(&self, importer: &mut Importer) -> AudioContentMetadata {
        let Self(ident_header, vorbis_comments) = &self;
        let channel_count = ChannelCount(ident_header.audio_channels.into());
        let channels = if channel_count.is_valid() {
            Some(channel_count.into())
        } else {
            importer.add_issue(format!(
                "Invalid number of channels: {num_channels}",
                num_channels = channel_count.0
            ));
            None
        };
        let bitrate = BitrateBps::from_inner(ident_header.bitrate_nominal.into());
        let bitrate = if bitrate.is_valid() {
            Some(bitrate)
        } else {
            importer.add_issue(format!("Invalid bitrate: {bitrate}"));
            None
        };
        let sample_rate = SampleRateHz::from_inner(ident_header.audio_sample_rate.into());
        let sample_rate = if sample_rate.is_valid() {
            Some(sample_rate)
        } else {
            importer.add_issue(format!("Invalid sample rate: {sample_rate}"));
            None
        };
        let loudness = vorbis::import_loudness(importer, vorbis_comments);
        let encoder = vorbis::import_encoder(vorbis_comments).map(Into::into);
        // TODO: The duration is not available from any header!?
        let duration = None;
        AudioContentMetadata {
            duration,
            channels,
            sample_rate,
            bitrate,
            loudness,
            encoder,
        }
    }

    pub fn import_into_track(
        self,
        importer: &mut Importer,
        config: &ImportTrackConfig,
        track: &mut Track,
    ) -> Result<()> {
        if track
            .media_source
            .content_metadata_flags
            .update(ContentMetadataFlags::RELIABLE)
        {
            let audio_content = self.import_audio_content(importer);
            track.media_source.content_metadata = ContentMetadata::Audio(audio_content);
        }

        let Self(_, vorbis_comments) = &self;
        vorbis::import_into_track(importer, vorbis_comments, config, track)
    }
}
