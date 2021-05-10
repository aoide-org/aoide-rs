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

use super::{schema::*, *};

use aoide_core::{
    collection::*,
    entity::{EntityHeader, EntityRevision},
    media::SourcePathKind,
    util::{clock::*, color::*},
};

use num_traits::FromPrimitive as _;
use url::Url;

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "collection"]
pub struct QueryableRecord {
    pub id: RowId,
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub entity_uid: Vec<u8>,
    pub entity_rev: i64,
    pub title: String,
    pub kind: Option<String>,
    pub notes: Option<String>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub media_source_path_kind: i16,
    pub media_source_base_url: Option<String>,
}

impl From<QueryableRecord> for (RecordHeader, Entity) {
    fn from(from: QueryableRecord) -> Self {
        let QueryableRecord {
            id,
            row_created_ms,
            row_updated_ms,
            entity_uid,
            entity_rev,
            title,
            kind,
            notes,
            color_rgb,
            color_idx,
            media_source_path_kind,
            media_source_base_url,
        } = from;
        let header = RecordHeader {
            id: id.into(),
            created_at: DateTime::new_timestamp_millis(row_created_ms),
            updated_at: DateTime::new_timestamp_millis(row_updated_ms),
        };
        let media_source_path_kind = SourcePathKind::from_i16(media_source_path_kind)
            .unwrap_or_else(|| {
                log::error!(
                    "Invalid media source path kind value: {}",
                    media_source_path_kind
                );
                SourcePathKind::Uri
            });
        let media_source_base_url = media_source_base_url
            .as_deref()
            .map(Url::parse)
            .transpose()
            .unwrap_or_else(|err| {
                log::error!(
                    "Invalid media source base URL '{}': {}",
                    media_source_base_url.unwrap_or_default(),
                    err,
                );
                None
            });
        let media_source_config = MediaSourceConfig {
            path_kind: media_source_path_kind,
            base_url: media_source_base_url,
        };
        let entity_hdr = entity_header_from_sql(&entity_uid, entity_rev);
        let entity_body = Collection {
            title,
            kind,
            notes,
            color: if let Some(color_rgb) = color_rgb {
                debug_assert!(color_idx.is_none());
                let rgb_color = RgbColor(color_rgb as RgbColorCode);
                debug_assert!(rgb_color.is_valid());
                Some(Color::Rgb(rgb_color))
            } else {
                color_idx.map(|idx| Color::Index(idx as ColorIndex))
            },
            media_source_config,
        };
        (header, Entity::new(entity_hdr, entity_body))
    }
}

impl From<QueryableRecord> for Entity {
    fn from(from: QueryableRecord) -> Self {
        let (_, entity) = from.into();
        entity
    }
}

#[derive(Debug, Insertable)]
#[table_name = "collection"]
pub struct InsertableRecord<'a> {
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub entity_uid: &'a [u8],
    pub entity_rev: i64,
    pub title: &'a str,
    pub kind: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub media_source_path_kind: i16,
    pub media_source_base_url: Option<&'a str>,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(created_at: DateTime, entity: &'a Entity) -> Self {
        let row_created_updated_ms = created_at.timestamp_millis();
        let (hdr, body) = entity.into();
        let EntityHeader { uid, rev } = hdr;
        let Collection {
            media_source_config:
                MediaSourceConfig {
                    path_kind: media_source_path_kind,
                    base_url: media_source_base_url,
                },
            title,
            kind,
            notes,
            color,
        } = body;
        Self {
            row_created_ms: row_created_updated_ms,
            row_updated_ms: row_created_updated_ms,
            entity_uid: uid.as_ref(),
            entity_rev: entity_revision_to_sql(*rev),
            title,
            kind: kind.as_deref(),
            notes: notes.as_deref(),
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
            media_source_path_kind: *media_source_path_kind as i16,
            media_source_base_url: media_source_base_url.as_ref().map(Url::as_str),
        }
    }
}

#[derive(Debug, AsChangeset)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "collection"]
pub struct TouchableRecord {
    pub row_updated_ms: TimestampMillis,
    pub entity_rev: i64,
}

impl TouchableRecord {
    pub fn bind(updated_at: DateTime, next_rev: EntityRevision) -> Self {
        let entity_rev = entity_revision_to_sql(next_rev);
        Self {
            row_updated_ms: updated_at.timestamp_millis(),
            entity_rev,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "collection"]
pub struct UpdatableRecord<'a> {
    pub row_updated_ms: TimestampMillis,
    pub entity_rev: i64,
    pub title: &'a str,
    pub kind: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub media_source_path_kind: i16,
    pub media_source_base_url: Option<&'a str>,
}

impl<'a> UpdatableRecord<'a> {
    pub fn bind(
        updated_at: DateTime,
        next_rev: EntityRevision,
        collection: &'a Collection,
    ) -> Self {
        let entity_rev = entity_revision_to_sql(next_rev);
        let Collection {
            media_source_config:
                MediaSourceConfig {
                    path_kind: media_source_path_kind,
                    base_url: media_source_base_url,
                },
            title,
            kind,
            notes,
            color,
        } = collection;
        Self {
            row_updated_ms: updated_at.timestamp_millis(),
            entity_rev,
            title,
            kind: kind.as_deref(),
            notes: notes.as_deref(),
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
            media_source_path_kind: *media_source_path_kind as i16,
            media_source_base_url: media_source_base_url.as_ref().map(Url::as_str),
        }
    }
}
