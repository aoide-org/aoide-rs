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

pub mod models;
pub mod schema;

use diesel::{query_builder::BoxedSelectStatement, sql_types::BigInt};

use aoide_repo::{collection::RecordId as CollectionId, media::source::RecordHeader};

use crate::prelude::*;

use self::schema::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ArtworkSource {
    Irregular = -2,
    Unsupported = -1,
    Missing = 0,
    Embedded = 1,
    Linked = 2,
}

impl ArtworkSource {
    pub fn try_read(value: i16) -> Option<Self> {
        let read = match value {
            0 => Self::Missing,
            1 => Self::Embedded,
            2 => Self::Linked,
            _ => return None,
        };
        Some(read)
    }

    pub const fn write(self) -> i16 {
        self as i16
    }
}

pub fn select_row_id_filtered_by_collection_id<'s, 'db, DB>(
    collection_id: CollectionId,
) -> BoxedSelectStatement<'db, BigInt, media_source::table, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    media_source::table
        .select(media_source::row_id)
        .filter(media_source::collection_id.eq(RowId::from(collection_id)))
        .into_boxed()
}

/// Filter by a path predicate.
///
/// URIs are only unambiguous within a collection. Therefore
/// filtering is restricted to a single collection.
pub fn select_row_id_filtered_by_content_path_predicate<'db, DB>(
    collection_id: CollectionId,
    content_path_predicate: StringPredicateBorrowed<'db>,
) -> BoxedSelectStatement<'db, BigInt, media_source::table, DB>
where
    DB: diesel::backend::Backend + 'db,
{
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
