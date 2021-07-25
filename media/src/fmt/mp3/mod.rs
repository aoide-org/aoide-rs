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

use minimp3::Decoder;

use aoide_core::{
    audio::{
        channel::{ChannelCount, NumberOfChannels},
        signal::SampleRateHz,
        AudioContent,
    },
    track::Track,
};

use crate::{
    io::import::{self, *},
    Result,
};

use super::id3::import_track as import_track_from_id3_tag;

#[derive(Debug)]
pub struct ImportTrack;

impl import::ImportTrack for ImportTrack {
    fn import_track(
        &self,
        config: &ImportTrackConfig,
        flags: ImportTrackFlags,
        track: Track,
        reader: &mut Box<dyn Reader>,
    ) -> Result<Track> {
        // Read number of channels and sample rate from the first decoded
        // MP3 frame. Those properties are supposed to be constant for the
        // whole MP3 file. Decoding the whole file would take too long.
        let mut decoder = Decoder::new(reader);
        let mut channels = None;
        let mut sample_rate = None;
        loop {
            let decoded_frame = decoder.next_frame();
            match decoded_frame {
                Ok(frame) => {
                    if frame.layer != 3
                        || frame.channels < 1
                        || frame.channels > 2
                        || frame.sample_rate <= 0
                        || frame.data.is_empty()
                    {
                        // Silently skip invalid or empty frames
                        log::warn!("Invalid MP3 frame: {:?}", frame);
                        continue;
                    }
                    channels = Some(ChannelCount(frame.channels as NumberOfChannels).into());
                    sample_rate = Some(SampleRateHz::from_inner(frame.sample_rate as f64));
                    // Stop decoding after receiving the first valid frame. Both the
                    // number of channels and the sample rate are supposed to be uniform
                    // for all frames!
                    break;
                }
                Err(minimp3::Error::Eof) => break,
                Err(minimp3::Error::Io(err)) => return Err(err.into()),
                Err(err) => return Err(anyhow::Error::from(err).into()),
            }
        }
        // Restore the reader
        let reader = decoder.into_inner();

        // Restart the reader for importing the exact duration and average bitrate
        let _start_pos = reader.seek(SeekFrom::Start(0))?;
        debug_assert_eq!(0, _start_pos);
        let duration = mp3_duration::from_read(reader).map(Into::into).ok();
        // TODO: Average bitrate needs to be calculated from all MP3 frames if
        // not stored explicitly. The mp3-duration crate already reads the bitrate
        // of each frame but does not calculate and return an average bitrate.
        let bitrate = None;

        let audio_content = AudioContent {
            duration,
            channels,
            sample_rate,
            bitrate,
            ..Default::default()
        };

        // Restart the reader for importing the ID3 tag
        let _start_pos = reader.seek(SeekFrom::Start(0))?;
        debug_assert_eq!(0, _start_pos);
        let id3_tag = id3::Tag::read_from(reader).map_err(anyhow::Error::from)?;
        import_track_from_id3_tag(config, flags, audio_content, track, &id3_tag)
    }
}
