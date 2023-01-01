// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use diesel::sql_types::BigInt;

use aoide_repo::{collection::RecordId as CollectionId, media::source::RecordHeader};

use crate::prelude::*;

use self::schema::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArtworkSource {
    Irregular = -2,
    Unsupported = -1,
    Missing = 0,
    Embedded = 1,
    Linked = 2,
}

impl ArtworkSource {
    fn try_read(value: i16) -> Option<Self> {
        let read = match value {
            0 => Self::Missing,
            1 => Self::Embedded,
            2 => Self::Linked,
            _ => return None,
        };
        Some(read)
    }

    const fn write(self) -> i16 {
        self as i16
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
    content_path_predicate: StringPredicateBorrowed<'_>,
) -> media_source::BoxedQuery<'_, DbBackend, BigInt> {
    // Source URI filtering
    let statement = media_source::table
        .select(media_source::row_id)
        .filter(media_source::collection_id.eq(RowId::from(collection_id)))
        .into_boxed();
    match content_path_predicate {
        StringPredicateBorrowed::StartsWith(path_prefix_nocase) => {
            if path_prefix_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path.like(escape_like_starts_with(path_prefix_nocase)),
            )
        }
        StringPredicateBorrowed::StartsNotWith(path_prefix_nocase) => {
            if path_prefix_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path
                    .not_like(escape_like_starts_with(path_prefix_nocase)),
            )
        }
        StringPredicateBorrowed::EndsWith(path_suffix_nocase) => {
            if path_suffix_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path.like(escape_like_ends_with(path_suffix_nocase)),
            )
        }
        StringPredicateBorrowed::EndsNotWith(path_suffix_nocase) => {
            if path_suffix_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path.not_like(escape_like_ends_with(path_suffix_nocase)),
            )
        }
        StringPredicateBorrowed::Contains(path_fragment_nocase) => {
            if path_fragment_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path.like(escape_like_contains(path_fragment_nocase)),
            )
        }
        StringPredicateBorrowed::ContainsNot(path_fragment_nocase) => {
            if path_fragment_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path
                    .not_like(escape_like_contains(path_fragment_nocase)),
            )
        }
        StringPredicateBorrowed::Matches(path_fragment_nocase) => {
            if path_fragment_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path.like(escape_like_matches(path_fragment_nocase)),
            )
        }
        StringPredicateBorrowed::MatchesNot(path_fragment_nocase) => {
            if path_fragment_nocase.is_empty() {
                return statement;
            }
            statement.filter(
                media_source::content_link_path.not_like(escape_like_matches(path_fragment_nocase)),
            )
        }
        StringPredicateBorrowed::Prefix(path_prefix) => {
            if path_prefix.is_empty() {
                return statement;
            }
            statement.filter(sql_column_substr_prefix_eq(
                "media_source.content_link_path",
                path_prefix,
            ))
        }
        StringPredicateBorrowed::Equals(path) => {
            statement.filter(media_source::content_link_path.eq(path))
        }
        StringPredicateBorrowed::EqualsNot(path) => {
            statement.filter(media_source::content_link_path.ne(path))
        }
    }
}
