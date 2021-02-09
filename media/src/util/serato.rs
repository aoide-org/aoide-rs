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

///////////////////////////////////////////////////////////////////////
use crate::Result;
use aoide_core::{
    audio::PositionMs,
    track::cue::{Cue, CueFlags},
    util::{
        color::{Color, RgbColor},
        CanonicalizeInto as _,
    },
};
use triseratops::tag::TagContainer;

/// Return a canonical vector of cues found in the tag container.
pub fn read_cues(serato_tags: &TagContainer) -> Result<Vec<Cue>> {
    let mut track_cues = vec![];

    for serato_cue in serato_tags.cues() {
        let color_code = (serato_cue.color.red as u32) << 16
            | (serato_cue.color.green as u32) << 8
            | serato_cue.color.blue as u32;

        let cue = Cue {
            bank_index: 0,
            slot_index: Some(serato_cue.index.into()),
            in_position: Some(PositionMs(serato_cue.position_millis.into())),
            out_position: None,
            out_mode: None,
            label: Some(serato_cue.label),
            color: Some(Color::Rgb(RgbColor(color_code))),
            flags: CueFlags::empty(),
        };
        track_cues.push(cue);
    }

    for serato_loop in serato_tags.loops() {
        let flags = if serato_loop.is_locked {
            CueFlags::LOCKED
        } else {
            CueFlags::empty()
        };
        let cue = Cue {
            bank_index: 1,
            slot_index: Some(serato_loop.index.into()),
            in_position: Some(PositionMs(serato_loop.start_position_millis.into())),
            out_position: Some(PositionMs(serato_loop.start_position_millis.into())),
            out_mode: None,
            label: Some(serato_loop.label),
            color: None,
            flags,
        };
        track_cues.push(cue);
    }

    let track_cues = track_cues.canonicalize_into();
    Ok(track_cues)
}
