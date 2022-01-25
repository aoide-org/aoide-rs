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

use super::{schema::*, *};

use aoide_core::{
    audio::{PositionInMilliseconds, PositionMs},
    track::cue::*,
    util::color::*,
};

use num_traits::{FromPrimitive, ToPrimitive};

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "track_cue"]
pub struct QueryableRecord {
    pub id: RowId,
    pub track_id: RowId,
    pub bank_idx: i16,
    pub slot_idx: Option<i16>,
    pub in_position_ms: Option<PositionInMilliseconds>,
    pub out_position_ms: Option<PositionInMilliseconds>,
    pub out_mode: Option<i16>,
    pub label: Option<String>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub flags: i16,
}

impl From<QueryableRecord> for (RecordId, Record) {
    fn from(from: QueryableRecord) -> Self {
        let QueryableRecord {
            id,
            track_id,
            bank_idx,
            slot_idx,
            in_position_ms,
            out_position_ms,
            out_mode,
            label,
            color_rgb,
            color_idx,
            flags,
        } = from;
        let in_marker = in_position_ms.map(|position_ms| InMarker {
            position: PositionMs(position_ms),
        });
        let out_marker = out_position_ms.map(|position_ms| OutMarker {
            position: PositionMs(position_ms),
            mode: out_mode.and_then(FromPrimitive::from_i16),
        });
        let cue = Cue {
            bank_index: bank_idx,
            slot_index: slot_idx,
            in_marker,
            out_marker,
            label,
            color: if let Some(color_rgb) = color_rgb {
                debug_assert!(color_idx.is_none());
                let rgb_color = RgbColor(color_rgb as RgbColorCode);
                debug_assert!(rgb_color.is_valid());
                Some(Color::Rgb(rgb_color))
            } else {
                color_idx.map(|idx| Color::Index(idx as ColorIndex))
            },
            flags: CueFlags::from_bits_truncate(flags as u8),
        };
        let record = Record {
            track_id: track_id.into(),
            cue,
        };
        (id.into(), record)
    }
}

#[derive(Debug, Insertable)]
#[table_name = "track_cue"]
pub struct InsertableRecord<'a> {
    pub track_id: RowId,
    pub bank_idx: BankIndex,
    pub slot_idx: Option<SlotIndex>,
    pub in_position_ms: Option<PositionInMilliseconds>,
    pub out_position_ms: Option<PositionInMilliseconds>,
    pub out_mode: Option<i16>,
    pub label: Option<&'a str>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub flags: i16,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(track_id: RecordId, cue: &'a Cue) -> Self {
        let Cue {
            bank_index,
            slot_index,
            in_marker,
            out_marker,
            label,
            color,
            flags,
        } = cue;
        let in_position = in_marker.as_ref().map(|InMarker { position }| position);
        let (out_position, out_mode) = out_marker
            .to_owned()
            .map(|OutMarker { position, mode }| (Some(position), mode))
            .unwrap_or((None, None));
        Self {
            track_id: track_id.into(),
            bank_idx: *bank_index,
            slot_idx: *slot_index,
            in_position_ms: in_position.map(|pos| pos.0),
            out_position_ms: out_position.map(|pos| pos.0),
            out_mode: out_mode.as_ref().and_then(ToPrimitive::to_i16),
            label: label.as_ref().map(String::as_str),
            color_rgb: if let Some(Color::Rgb(color)) = color {
                Some(color.code() as i32)
            } else {
                None
            },
            color_idx: if let Some(Color::Index(index)) = color {
                Some(*index)
            } else {
                None
            },
            flags: flags.bits() as i16,
        }
    }
}
