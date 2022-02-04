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

use std::{borrow::Cow, io::SeekFrom, path::Path, time::Duration};

use id3::TagLike as _;
use mpeg_audio_header::{Header, ParseMode};

use aoide_core::{
    audio::{
        channel::{ChannelCount, NumberOfChannels},
        signal::{BitrateBps, BitsPerSecond, SampleRateHz, SamplesPerSecond},
        AudioContent,
    },
    media::{ApicType, Content, ContentMetadataFlags},
    track::Track,
};

use crate::{
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, Importer, Reader},
    },
    Error, Result,
};

use super::id3::{
    export_track as export_track_into_id3_tag, import_encoder, import_loudness,
    import_metadata_into_track, map_id3_err,
};

fn mpeg_audio_header_error(err: mpeg_audio_header::PositionalError) -> Error {
    anyhow::Error::from(err).into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata(id3::Tag);

impl Metadata {
    pub fn read_from(reader: &mut impl Reader) -> Result<Option<Self>> {
        match id3::Tag::read_from(reader) {
            Ok(id3_tag) => Ok(Some(Self(id3_tag))),
            Err(err) => match err.kind {
                id3::ErrorKind::NoTag => Ok(None),
                _ => Err(map_id3_err(err)),
            },
        }
    }

    #[must_use]
    pub fn find_embedded_artwork_image(&self) -> Option<(ApicType, &str, &[u8])> {
        let Self(id3_tag) = self;
        super::id3::find_embedded_artwork_image(id3_tag)
    }
}

#[derive(Debug, Clone)]
pub struct MetadataExt(Header, Option<Metadata>);

impl MetadataExt {
    pub fn read_from(reader: &mut impl Reader) -> Result<Self> {
        let stream_info = Header::read_from_source(reader, ParseMode::IgnoreVbrHeaders)
            .map_err(mpeg_audio_header_error)?;
        // Restart the reader for importing the ID3 tag
        let _start_pos = reader.seek(SeekFrom::Start(0))?;
        debug_assert_eq!(0, _start_pos);
        let metadata = Metadata::read_from(reader)?;
        Ok(Self(stream_info, metadata))
    }

    #[must_use]
    pub fn import_audio_content(&self, importer: &mut Importer) -> AudioContent {
        let Self(stream_info, metadata) = self;
        let Header {
            max_channel_count,
            avg_sample_rate_hz,
            avg_bitrate_bps,
            total_duration,
            ..
        } = stream_info;
        let loudness;
        let encoder;
        if let Some(Metadata(id3_tag)) = metadata {
            loudness = import_loudness(importer, id3_tag);
            encoder = import_encoder(id3_tag).map(Cow::into_owned);
        } else {
            loudness = None;
            encoder = None;
        }
        AudioContent {
            duration: Some(total_duration.to_owned().into()),
            channels: Some(ChannelCount(*max_channel_count as NumberOfChannels).into()),
            sample_rate: avg_sample_rate_hz
                .map(|hz| SampleRateHz::from_inner(SamplesPerSecond::from(hz))),
            bitrate: avg_bitrate_bps.map(|bps| BitrateBps::from_inner(BitsPerSecond::from(bps))),
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
        let mut audio_content = self.import_audio_content(importer);
        let id3_tag = if let Self(_, Some(Metadata(id3_tag))) = self {
            id3_tag
        } else {
            // No ID3 tag available
            return Ok(());
        };
        let update_metadata_flags = if audio_content.duration.is_some() {
            // Accurate duration
            ContentMetadataFlags::RELIABLE
        } else {
            audio_content.duration = id3_tag
                .duration()
                .map(|secs| Duration::from_secs(u64::from(secs)).into());
            ContentMetadataFlags::UNRELIABLE
        };
        if track
            .media_source
            .content_metadata_flags
            .update(update_metadata_flags)
        {
            track.media_source.content = Content::Audio(audio_content);
        } else {
            log::info!(
                "Skipping import of audio content for {}",
                track.media_source.path
            );
        }

        import_metadata_into_track(importer, &id3_tag, config, track)
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
