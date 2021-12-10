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

use std::{io::SeekFrom, path::Path};

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
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, Reader},
    },
    Error, Result,
};

use super::id3::{
    export_track as export_track_into_id3_tag, import_track as import_track_from_id3_tag,
    map_err as map_id3_err,
};

fn map_mp3_duration_err(err: mp3_duration::MP3DurationError) -> Error {
    anyhow::Error::from(err).into()
}

pub type Tag = id3::Tag;

pub fn read_tag_from(reader: &mut impl Reader) -> Result<Tag> {
    id3::Tag::read_from(reader).map_err(map_id3_err)
}

pub fn import_track(
    reader: &mut Box<dyn Reader>,
    config: &ImportTrackConfig,
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
    let id3_tag = read_tag_from(reader).map_err(|err| {
        tracing::warn!(
            "Failed to parse ID3 tag from media source '{}': {}",
            track.media_source.path,
            err
        );
        err
    })?;
    import_track_from_id3_tag(&id3_tag, audio_content, config, track)
}

pub fn export_track_to_path(
    path: &Path,
    config: &ExportTrackConfig,
    track: &mut Track,
) -> Result<bool> {
    let id3_tag_orig = id3::Tag::read_from_path(path).map_err(map_id3_err)?;

    let mut id3_tag = id3_tag_orig.clone();
    export_track_into_id3_tag(config, track, &mut id3_tag)
        .map_err(|err| Error::Other(anyhow::anyhow!("Failed to export ID3 tag: {:?}", err)))?;

    if id3_tag == id3_tag_orig {
        // Unmodified
        return Ok(false);
    }
    id3_tag
        .write_to_path(path, id3::Version::Id3v24)
        .map_err(map_id3_err)?;
    // Modified
    Ok(true)
}
