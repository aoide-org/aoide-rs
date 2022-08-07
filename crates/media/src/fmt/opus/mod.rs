// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use opus_headers::{CommentHeader, OpusHeaders, ParseError as OpusError};

use aoide_core::{
    audio::{channel::ChannelCount, signal::SampleRateHz},
    media::{
        artwork::ApicType,
        content::{AudioContentMetadata, ContentMetadata, ContentMetadataFlags},
    },
    track::Track,
};

use crate::{io::import::*, Error, Result};

use super::vorbis;

fn map_opus_err(err: OpusError) -> Error {
    match err {
        OpusError::Io(err) => Error::Io(err),
        err => Error::Other(anyhow::Error::from(err)),
    }
}

#[allow(missing_debug_implementations)]
pub struct Metadata(OpusHeaders);

impl Metadata {
    pub fn read_from(reader: &mut impl Reader) -> Result<Self> {
        opus_headers::parse_from_read(reader)
            .map(Self)
            .map_err(map_opus_err)
    }

    #[must_use]
    pub fn find_embedded_artwork_image(
        &self,
        importer: &mut Importer,
    ) -> Option<(ApicType, String, Vec<u8>)> {
        let Self(OpusHeaders {
            id: _,
            comments:
                CommentHeader {
                    vendor: _,
                    user_comments,
                },
        }) = self;
        vorbis::find_embedded_artwork_image(importer, user_comments)
    }

    pub fn import_audio_content(&self, importer: &mut Importer) -> AudioContentMetadata {
        let Self(OpusHeaders {
            id: id_header,
            comments:
                CommentHeader {
                    vendor: _,
                    user_comments,
                },
        }) = self;
        let channel_count = ChannelCount(id_header.channel_count.into());
        let channels = if channel_count.is_valid() {
            Some(channel_count.into())
        } else {
            importer.add_issue(format!(
                "Invalid number of channels: {num_channels}",
                num_channels = channel_count.0
            ));
            None
        };
        let bitrate = None;
        let sample_rate = SampleRateHz::from_inner(id_header.input_sample_rate.into());
        let sample_rate = if sample_rate.is_valid() {
            Some(sample_rate)
        } else {
            importer.add_issue(format!("Invalid sample rate: {sample_rate}"));
            None
        };
        let loudness = vorbis::import_loudness(importer, user_comments);
        let encoder = vorbis::import_encoder(user_comments).map(Into::into);
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
            .content
            .metadata_flags
            .update(ContentMetadataFlags::RELIABLE)
        {
            let audio_content = self.import_audio_content(importer);
            track.media_source.content.metadata = ContentMetadata::Audio(audio_content);
        }

        let Self(OpusHeaders {
            id: _,
            comments:
                CommentHeader {
                    vendor: _,
                    user_comments,
                },
        }) = self;
        vorbis::import_into_track(importer, &user_comments, config, track)
    }
}
