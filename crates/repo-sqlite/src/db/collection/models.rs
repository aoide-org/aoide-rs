// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    collection::MediaSourceConfig,
    media::content::{ContentPathConfig, ContentPathKind},
    util::{clock::*, color::*, url::BaseUrl},
    Collection, CollectionEntity, CollectionHeader, EntityRevision,
};
use num_traits::FromPrimitive as _;
use url::Url;

use super::{schema::*, *};

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = collection)]
pub struct QueryableRecord {
    pub id: RowId,
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub entity_uid: String,
    pub entity_rev: i64,
    pub title: String,
    pub kind: Option<String>,
    pub notes: Option<String>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub media_source_path_kind: i16,
    pub media_source_root_url: Option<String>,
}

impl TryFrom<QueryableRecord> for (RecordHeader, CollectionEntity) {
    type Error = anyhow::Error;

    fn try_from(from: QueryableRecord) -> anyhow::Result<Self> {
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
            media_source_root_url,
        } = from;
        let header = RecordHeader {
            id: id.into(),
            created_at: DateTime::new_timestamp_millis(row_created_ms),
            updated_at: DateTime::new_timestamp_millis(row_updated_ms),
        };
        let Some(media_source_path_kind) = ContentPathKind::from_i16(media_source_path_kind) else {
            anyhow::bail!("Invalid media source path kind value: {media_source_path_kind}");
        };
        let media_source_root_url = media_source_root_url
            .as_deref()
            .map(BaseUrl::parse_strict)
            .transpose()?;
        let content_path_config =
            ContentPathConfig::try_from((media_source_path_kind, media_source_root_url))?;
        let media_source_config = MediaSourceConfig {
            content_path: content_path_config,
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
        let entity = CollectionEntity::new(CollectionHeader::from_untyped(entity_hdr), entity_body);
        Ok((header, entity))
    }
}

impl TryFrom<QueryableRecord> for CollectionEntity {
    type Error = anyhow::Error;

    fn try_from(from: QueryableRecord) -> anyhow::Result<Self> {
        let (_, entity) = from.try_into()?;
        Ok(entity)
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = collection)]
pub struct InsertableRecord<'a> {
    pub row_created_ms: TimestampMillis,
    pub row_updated_ms: TimestampMillis,
    pub entity_uid: String,
    pub entity_rev: i64,
    pub title: &'a str,
    pub kind: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub media_source_path_kind: i16,
    pub media_source_root_url: Option<&'a str>,
}

impl<'a> InsertableRecord<'a> {
    pub fn bind(created_at: DateTime, entity: &'a CollectionEntity) -> Self {
        let row_created_updated_ms = created_at.timestamp_millis();
        let (hdr, body) = entity.into();
        let CollectionHeader { uid, rev } = hdr;
        let Collection {
            media_source_config:
                MediaSourceConfig {
                    content_path: content_path_config,
                },
            title,
            kind,
            notes,
            color,
        } = body;
        let media_source_path_kind = content_path_config.kind();
        let media_source_root_url: Option<&Url> = content_path_config.root_url().map(Deref::deref);
        Self {
            row_created_ms: row_created_updated_ms,
            row_updated_ms: row_created_updated_ms,
            entity_uid: entity_uid_to_sql(uid),
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
            media_source_path_kind: media_source_path_kind as i16,
            media_source_root_url: media_source_root_url.map(Url::as_str),
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = collection, treat_none_as_null = true)]
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
#[diesel(table_name = collection, treat_none_as_null = true)]
pub struct UpdatableRecord<'a> {
    pub row_updated_ms: TimestampMillis,
    pub entity_rev: i64,
    pub title: &'a str,
    pub kind: Option<&'a str>,
    pub notes: Option<&'a str>,
    pub color_rgb: Option<i32>,
    pub color_idx: Option<i16>,
    pub media_source_path_kind: i16,
    pub media_source_root_url: Option<&'a str>,
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
                    content_path: content_path_config,
                },
            title,
            kind,
            notes,
            color,
        } = collection;
        let media_source_path_kind = content_path_config.kind();
        let media_source_root_url: Option<&Url> = content_path_config.root_url().map(Deref::deref);
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
            media_source_path_kind: media_source_path_kind as i16,
            media_source_root_url: media_source_root_url.map(Url::as_str),
        }
    }
}
