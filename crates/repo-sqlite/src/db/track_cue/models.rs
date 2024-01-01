// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    audio::{PositionMs, PositionMsValue},
    track::cue::*,
    util::color::*,
};

use super::{schema::*, *};

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = track_cue, primary_key(row_id))]
pub struct QueryableRecord {
    pub row_id: RowId,
    pub track_id: RowId,
    pub bank_idx: i16,
    pub slot_idx: Option<i16>,
    pub in_position_ms: Option<PositionMsValue>,
    pub out_position_ms: Option<PositionMsValue>,
    pub out_mode: Option<i16>,
    pub kind: Option<String>,
    pub label: Option<String>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub flags: i16,
}

impl From<QueryableRecord> for (RecordId, Record) {
    fn from(from: QueryableRecord) -> Self {
        let QueryableRecord {
            row_id,
            track_id,
            bank_idx,
            slot_idx,
            in_position_ms,
            out_position_ms,
            out_mode,
            kind,
            label,
            color_rgb,
            color_idx,
            flags,
        } = from;
        let in_marker = in_position_ms.map(|position_ms| InMarker {
            position: PositionMs::new(position_ms),
        });
        let out_marker = out_position_ms.map(|position_ms| OutMarker {
            position: PositionMs::new(position_ms),
            mode: out_mode
                .map(TryInto::try_into)
                .transpose()
                .ok()
                .flatten()
                .and_then(OutMode::from_repr),
        });
        let cue = Cue {
            bank_index: bank_idx,
            slot_index: slot_idx,
            in_marker,
            out_marker,
            kind,
            label,
            color: if let Some(color_rgb) = color_rgb {
                debug_assert!(color_idx.is_none());
                let rgb_color = RgbColor::new(color_rgb as RgbColorCode);
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
        (row_id.into(), record)
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = track_cue)]
pub struct InsertableRecord<'a> {
    pub track_id: RowId,
    pub bank_idx: BankIndex,
    pub slot_idx: Option<SlotIndex>,
    pub in_position_ms: Option<PositionMsValue>,
    pub out_position_ms: Option<PositionMsValue>,
    pub out_mode: Option<i16>,
    pub kind: Option<&'a str>,
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
            kind,
            label,
            color,
            flags,
        } = cue;
        let in_position = in_marker.as_ref().map(|InMarker { position }| *position);
        let (out_position, out_mode) = out_marker
            .clone()
            .map_or((None, None), |OutMarker { position, mode }| {
                (Some(position), mode)
            });
        Self {
            track_id: track_id.into(),
            bank_idx: *bank_index,
            slot_idx: *slot_index,
            in_position_ms: in_position.map(PositionMs::value),
            out_position_ms: out_position.map(PositionMs::value),
            out_mode: out_mode.map(|out_mode| out_mode as _),
            kind: kind.as_ref().map(String::as_str),
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
            flags: i16::from(flags.bits()),
        }
    }
}
