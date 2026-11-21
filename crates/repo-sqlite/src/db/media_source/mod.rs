// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use diesel::{prelude::*, sql_types::BigInt};
use strum::FromRepr;

use aoide_core::media::{artwork::ApicType, content::ContentPathKind};
use aoide_core_api::filter::{StringCompare, StringPredicate};
use aoide_repo::{CollectionId, RepoResult};

use crate::{
    DbBackend, RowId,
    util::{
        LIKE_ESCAPE_CHARACTER, StringCmpOp, decode_repr, decompose_string_predicate,
        sql_column_substr_prefix_eq, sql_column_substr_prefix_ne,
    },
};

use self::schema::*;

pub(crate) const fn encode_content_path_kind(value: ContentPathKind) -> i16 {
    value as _
}

pub(crate) fn decode_content_path_kind(value: i16) -> RepoResult<ContentPathKind> {
    decode_repr(value, ContentPathKind::from_repr, "ContentPathKind")
}

pub(crate) const fn encode_apic_type(value: ApicType) -> i16 {
    value as _
}

pub(crate) fn decode_apic_type(value: i16) -> RepoResult<ApicType> {
    decode_repr(value, ApicType::from_repr, "ApicType")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(i8)]
enum ArtworkSource {
    Irregular = -2,
    Unsupported = -1,
    Missing = 0,
    Embedded = 1,
    Linked = 2,
}

impl ArtworkSource {
    fn decode(value: i16) -> RepoResult<Self> {
        decode_repr(value, Self::from_repr, "ArtworkSource")
    }

    const fn encode(self) -> i16 {
        self as _
    }
}

pub(crate) fn select_row_id_filtered_by_collection_id<'db>(
    collection_id: CollectionId,
) -> media_source::BoxedQuery<'db, DbBackend, BigInt> {
    media_source::table
        .select(media_source::row_id)
        .filter(media_source::collection_id.eq(RowId::from(collection_id)))
        .into_boxed()
}

/// Filter by a path predicate.
///
/// URIs are only unambiguous within a collection. Therefore
/// filtering is restricted to a single collection.
pub(crate) fn select_row_id_filtered_by_content_path_predicate(
    collection_id: CollectionId,
    content_path_predicate: &StringPredicate<'_>,
) -> media_source::BoxedQuery<'static, DbBackend, BigInt> {
    // Source URI filtering
    let statement = media_source::table
        .select(media_source::row_id)
        .filter(media_source::collection_id.eq(RowId::from(collection_id)))
        .into_boxed();
    let Some((val, cmp, dir)) = decompose_string_predicate(content_path_predicate) else {
        return statement;
    };
    if val.is_empty() && cmp != StringCompare::Equals {
        // Include all
        return statement;
    }
    match StringCmpOp::new(cmp, val) {
        StringCmpOp::Equal(path) => {
            if dir {
                statement.filter(media_source::content_link_path.eq(path))
            } else {
                statement.filter(media_source::content_link_path.ne(path))
            }
        }
        StringCmpOp::Prefix(path_prefix) => {
            let column = "media_source.content_link_path";
            if dir {
                statement.filter(sql_column_substr_prefix_eq(column, &path_prefix))
            } else {
                statement.filter(sql_column_substr_prefix_ne(column, &path_prefix))
            }
        }
        StringCmpOp::Like { escaped } => {
            if dir {
                statement.filter(
                    media_source::content_link_path
                        .like(escaped)
                        .escape(LIKE_ESCAPE_CHARACTER),
                )
            } else {
                statement.filter(
                    media_source::content_link_path
                        .not_like(escaped)
                        .escape(LIKE_ESCAPE_CHARACTER),
                )
            }
        }
    }
}
