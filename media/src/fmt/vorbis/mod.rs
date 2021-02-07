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

use crate::util::{
    parse_index_numbers, parse_key_signature, parse_replay_gain, parse_tempo_bpm, parse_year_tag,
    tag::{import_faceted_tags, FacetedTagMappingConfig},
};

use aoide_core::{
    audio::signal::LoudnessLufs,
    media::concat_encoder_properties,
    music::{key::KeySignature, time::TempoBpm},
    tag::{Facet, Score as TagScore, TagsMap},
    track::{
        album::AlbumKind,
        index::Index,
        release::DateOrDateTime,
        title::{Title, TitleKind},
    },
    util::CanonicalizeInto as _,
};

use semval::IsValid as _;
use std::borrow::Cow;

pub trait CommentReader {
    fn read_first_value(&self, key: &str) -> Option<&str>;
}

pub fn import_faceted_text_tags<'a>(
    tags_map: &mut TagsMap,
    config: &FacetedTagMappingConfig,
    facet: &Facet,
    label_iter: impl Iterator<Item = &'a str>,
) {
    let removed_tags = tags_map.remove_faceted_tags(&facet);
    if removed_tags > 0 {
        log::debug!("Replacing {} custom '{}' tags", removed_tags, facet.value());
    }
    let tag_mapping_config = config.get(facet.value());
    let mut next_score_value = TagScore::max_value();
    for label in label_iter {
        import_faceted_tags(
            tags_map,
            &mut next_score_value,
            &facet,
            tag_mapping_config,
            label,
        );
    }
}

pub fn import_loudness(reader: &impl CommentReader) -> Option<LoudnessLufs> {
    reader
        .read_first_value("REPLAYGAIN_TRACK_GAIN")
        .and_then(parse_replay_gain)
}

pub fn import_encoder<'a>(reader: &'a impl CommentReader) -> Option<Cow<'a, str>> {
    concat_encoder_properties(
        reader.read_first_value("ENCODEDBY"),
        reader.read_first_value("ENCODERSETTINGS"),
    )
}

pub fn import_tempo_bpm(reader: &impl CommentReader) -> Option<TempoBpm> {
    if let Some(tempo_bpm) = reader
        .read_first_value("BPM")
        .and_then(parse_tempo_bpm)
        // Alternative: Try "TEMPO" if "BPM" is missing or invalid
        .or_else(|| reader.read_first_value("TEMPO").and_then(parse_tempo_bpm))
    {
        debug_assert!(tempo_bpm.is_valid());
        Some(tempo_bpm)
    } else {
        None
    }
}

pub fn import_key_signature(reader: &impl CommentReader) -> Option<KeySignature> {
    reader
        .read_first_value("INITIALKEY")
        .and_then(parse_key_signature)
        .or_else(|| reader.read_first_value("KEY").and_then(parse_key_signature))
}

pub fn import_album_kind(reader: &impl CommentReader) -> Option<AlbumKind> {
    if reader
        .read_first_value("COMPILATION")
        .and_then(|compilation| compilation.parse::<u8>().ok())
        .unwrap_or_default()
        == 1
    {
        Some(AlbumKind::Compilation)
    } else {
        None
    }
}

pub fn import_released_at(reader: &impl CommentReader) -> Option<DateOrDateTime> {
    reader.read_first_value("DATE").and_then(parse_year_tag)
}

pub fn import_released_by(reader: &impl CommentReader) -> Option<String> {
    reader.read_first_value("LABEL").map(ToOwned::to_owned)
}

pub fn import_release_copyright(reader: &impl CommentReader) -> Option<String> {
    reader.read_first_value("COPYRIGHT").map(ToOwned::to_owned)
}

pub fn import_track_index(reader: &impl CommentReader) -> Option<Index> {
    if let Some(mut index) = reader
        .read_first_value("TRACKNUMBER")
        .and_then(parse_index_numbers)
    {
        if index.total.is_none() {
            // According to https://wiki.xiph.org/Field_names "TRACKTOTAL" is
            // the proposed field name, but some applications use "TOTALTRACKS".
            index.total = reader
                .read_first_value("TRACKTOTAL")
                .and_then(|input| input.parse().ok())
                .or_else(|| {
                    reader
                        .read_first_value("TOTALTRACKS")
                        .and_then(|input| input.parse().ok())
                });
        }
        Some(index)
    } else {
        None
    }
}

pub fn import_disc_index(reader: &impl CommentReader) -> Option<Index> {
    if let Some(mut index) = reader
        .read_first_value("DISCNUMBER")
        .and_then(parse_index_numbers)
    {
        if index.total.is_none() {
            // According to https://wiki.xiph.org/Field_names "DISCTOTAL" is
            // the proposed field name, but some applications use "TOTALDISCS".
            index.total = reader
                .read_first_value("DISCTOTAL")
                .and_then(|input| input.parse().ok())
                .or_else(|| {
                    reader
                        .read_first_value("TOTALDISCS")
                        .and_then(|input| input.parse().ok())
                });
        }
        Some(index)
    } else {
        None
    }
}

pub fn import_movement_index(reader: &impl CommentReader) -> Option<Index> {
    if let Some(mut index) = reader
        .read_first_value("MOVEMENT")
        .and_then(parse_index_numbers)
    {
        if index.total.is_none() {
            index.total = reader
                .read_first_value("MOVEMENTTOTAL")
                .and_then(|input| input.parse().ok());
        }
        Some(index)
    } else {
        None
    }
}

pub fn import_track_titles(reader: &impl CommentReader) -> Vec<Title> {
    let mut track_titles = Vec::with_capacity(4);
    if let Some(name) = reader.read_first_value("TITLE") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Main,
        };
        track_titles.push(title);
    }
    if let Some(name) = reader.read_first_value("SUBTITLE") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Sub,
        };
        track_titles.push(title);
    }
    if let Some(name) = reader.read_first_value("WORK") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Work,
        };
        track_titles.push(title);
    }
    if let Some(name) = reader.read_first_value("MOVEMENTNAME") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Movement,
        };
        track_titles.push(title);
    }
    track_titles.canonicalize_into()
}

pub fn import_album_titles(reader: &impl CommentReader) -> Vec<Title> {
    let mut album_titles = Vec::with_capacity(1);
    if let Some(name) = reader.read_first_value("ALBUM") {
        let title = Title {
            name: name.to_owned(),
            kind: TitleKind::Main,
        };
        album_titles.push(title);
    }
    album_titles.canonicalize_into()
}
