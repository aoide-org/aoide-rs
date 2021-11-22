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

use std::io::SeekFrom;

use aoide_core::{
    audio::{
        channel::{ChannelCount, NumberOfChannels},
        signal::{BitrateBps, BitsPerSecond, SampleRateHz, SamplesPerSecond},
        AudioContent,
    },
    track::Track,
};
use mp3_duration::{ParseMode, StreamInfo};

use crate::{
    io::import::{self, *},
    Error, Result,
};

use super::id3::{import_track as import_track_from_id3_tag, map_err as map_id3_err};

fn map_mp3_duration_err(err: mp3_duration::MP3DurationError) -> Error {
    anyhow::Error::from(err).into()
}

#[derive(Debug)]
pub struct ImportTrack;

impl import::ImportTrack for ImportTrack {
    fn import_track(
        &self,
        reader: &mut Box<dyn Reader>,
        config: &ImportTrackConfig,
        flags: ImportTrackFlags,
        track: &mut Track,
    ) -> Result<()> {
        let audio_content = StreamInfo::read(reader, ParseMode::Exact)
            .map(|stream_info| {
                let StreamInfo {
                    max_channel_count,
                    avg_sampling_rate,
                    avg_bitrate,
                    duration,
                    ..
                } = stream_info;
                AudioContent {
                    duration: Some(duration.into()),
                    channels: Some(ChannelCount(max_channel_count as NumberOfChannels).into()),
                    sample_rate: Some(SampleRateHz::from_inner(
                        avg_sampling_rate as SamplesPerSecond,
                    )),
                    bitrate: Some(BitrateBps::from_inner(avg_bitrate as BitsPerSecond)),
                    ..Default::default()
                }
            })
            .map_err(|err| {
                tracing::warn!(
                    "Failed to parse audio properties from media source '{}': {}",
                    track.media_source.path,
                    err
                );
                map_mp3_duration_err(err)
            })?;

        // Restart the reader after importing the stream info
        let _start_pos = reader.seek(SeekFrom::Start(0))?;
        debug_assert_eq!(0, _start_pos);

        // Restart the reader for importing the ID3 tag
        let _start_pos = reader.seek(SeekFrom::Start(0))?;
        debug_assert_eq!(0, _start_pos);
        let id3_tag = id3::Tag::read_from(reader).map_err(|err| {
            tracing::warn!(
                "Failed to parse ID3 tag from media source '{}': {}",
                track.media_source.path,
                err
            );
            map_id3_err(err)
        })?;
        import_track_from_id3_tag(config, flags, audio_content, track, &id3_tag)
    }
}
