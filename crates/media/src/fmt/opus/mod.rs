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

use opus_headers::{CommentHeader, OpusHeaders, ParseError as OpusError};
use semval::IsValid as _;

use aoide_core::{
    audio::{channel::ChannelCount, signal::SampleRateHz, AudioContent},
    media::{ApicType, Content, ContentMetadataFlags},
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

    pub fn import_audio_content(&self, importer: &mut Importer) -> AudioContent {
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
            importer.add_issue(format!("Invalid channel count: {}", channel_count.0));
            None
        };
        let bitrate = None;
        let sample_rate = SampleRateHz::from_inner(id_header.input_sample_rate.into());
        let sample_rate = if sample_rate.is_valid() {
            Some(sample_rate)
        } else {
            importer.add_issue(format!("Invalid sample rate: {}", sample_rate));
            None
        };
        let loudness = vorbis::import_loudness(importer, user_comments);
        let encoder = vorbis::import_encoder(user_comments).map(Into::into);
        // TODO: The duration is not available from any header!?
        let duration = None;
        AudioContent {
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
            track.media_source.content = Content::Audio(audio_content);
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
