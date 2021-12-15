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

use std::{io::SeekFrom, ops::Deref, path::Path};

use aoide_core::{
    audio::{
        channel::{ChannelCount, NumberOfChannels},
        signal::{BitrateBps, BitsPerSecond, SampleRateHz, SamplesPerSecond},
        AudioContent,
    },
    media::ApicType,
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
    export_track as export_track_into_id3_tag, import_metadata_into_track, map_id3_err,
};

fn map_mp3_duration_err(err: mp3_duration::MP3DurationError) -> Error {
    anyhow::Error::from(err).into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata(id3::Tag);

impl Metadata {
    pub fn read_from(reader: &mut impl Reader) -> Result<Self> {
        id3::Tag::read_from(reader).map(Self).map_err(map_id3_err)
    }

    pub fn find_embedded_artwork_image(&self) -> Option<(ApicType, &str, &[u8])> {
        let Self(id3_tag) = self;
        super::id3::find_embedded_artwork_image(id3_tag)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataExt(StreamInfo, Metadata);

impl AsRef<Metadata> for MetadataExt {
    fn as_ref(&self) -> &Metadata {
        let Self(_, metadata) = self;
        metadata
    }
}

impl Deref for MetadataExt {
    type Target = Metadata;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl MetadataExt {
    pub fn read_from(reader: &mut impl Reader) -> Result<Self> {
        let stream_info =
            StreamInfo::read(reader, ParseMode::Exact).map_err(map_mp3_duration_err)?;
        // Restart the reader for importing the ID3 tag
        let _start_pos = reader.seek(SeekFrom::Start(0))?;
        debug_assert_eq!(0, _start_pos);
        let metadata = Metadata::read_from(reader)?;
        Ok(Self(stream_info, metadata))
    }

    pub fn import_into_track(self, config: &ImportTrackConfig, track: &mut Track) -> Result<()> {
        let Self(stream_info, metadata) = self;
        let Metadata(id3_tag) = metadata;

        let StreamInfo {
            max_channel_count,
            avg_sampling_rate,
            avg_bitrate,
            duration,
            ..
        } = stream_info;
        let audio_content = AudioContent {
            duration: Some(duration.into()),
            channels: Some(ChannelCount(max_channel_count as NumberOfChannels).into()),
            sample_rate: Some(SampleRateHz::from_inner(
                avg_sampling_rate as SamplesPerSecond,
            )),
            bitrate: Some(BitrateBps::from_inner(avg_bitrate as BitsPerSecond)),
            ..Default::default()
        };

        import_metadata_into_track(audio_content, &id3_tag, config, track)
    }
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
