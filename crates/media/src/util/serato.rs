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

use aoide_core::{
    audio::PositionMs,
    track::cue::{Cue, CueFlags, InMarker, OutMarker, OutMode},
    util::{
        canonical::CanonicalizeInto as _,
        color::{Color, RgbColor},
        string::trimmed_non_empty_from_owned,
    },
};

use triseratops::tag::{
    color::Color as SeratoColor,
    generic::{Cue as SeratoCue, Loop},
    TagContainer,
};

const CUE_BANK_INDEX: i16 = 1;
const LOOP_BANK_INDEX: i16 = 2;

fn import_cue(serato_cue: SeratoCue) -> Cue {
    Cue {
        bank_index: CUE_BANK_INDEX,
        slot_index: Some(serato_cue.index.into()),
        in_marker: Some(InMarker {
            position: PositionMs(serato_cue.position.millis.into()),
        }),
        out_marker: None,
        label: trimmed_non_empty_from_owned(serato_cue.label).map(Into::into),
        color: Some(Color::Rgb(RgbColor(
            serato_cue.color.into_pro_hotcue_color().into(),
        ))),
        flags: CueFlags::empty(),
    }
}

fn import_loop(serato_loop: Loop) -> Cue {
    let flags = if serato_loop.is_locked {
        CueFlags::LOCKED
    } else {
        CueFlags::empty()
    };
    Cue {
        bank_index: LOOP_BANK_INDEX,
        slot_index: Some(serato_loop.index.into()),
        in_marker: Some(InMarker {
            position: PositionMs(serato_loop.start_position.millis.into()),
        }),
        out_marker: Some(OutMarker {
            position: PositionMs(serato_loop.end_position.millis.into()),
            mode: Some(OutMode::Loop),
        }),
        label: trimmed_non_empty_from_owned(serato_loop.label).map(Into::into),
        color: None,
        flags,
    }
}

/// Return a canonical vector of cues found in the tag container.
#[must_use]
pub fn import_cues(serato_tags: &TagContainer) -> Vec<Cue> {
    serato_tags
        .cues()
        .into_iter()
        .map(import_cue)
        .chain(serato_tags.loops().into_iter().map(import_loop))
        .collect::<Vec<_>>()
        .canonicalize_into()
}

pub fn import_track_color(serato_tags: &TagContainer) -> Option<Color> {
    serato_tags
        .track_color()
        .and_then(SeratoColor::into_displayed_track_color)
        .map(Into::into)
        .map(RgbColor)
        .map(Color::Rgb)
}
