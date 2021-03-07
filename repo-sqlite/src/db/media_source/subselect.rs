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

use super::schema::*;

use crate::prelude::*;

use aoide_repo::collection::RecordId as CollectionId;
use diesel::{query_builder::BoxedSelectStatement, sql_types::BigInt};

pub fn filter_by_collection_id<'s, 'db, DB>(
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
pub fn filter_by_path_predicate<'db, DB>(
    collection_id: CollectionId,
    path_predicate: StringPredicateBorrowed<'db>,
) -> BoxedSelectStatement<'db, BigInt, media_source::table, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    // Source URI filtering
    let statement = media_source::table
        .select(media_source::row_id)
        .filter(media_source::collection_id.eq(RowId::from(collection_id)))
        .into_boxed();
    match path_predicate {
        StringPredicateBorrowed::StartsWith(path_prefix_nocase) => {
            statement.filter(media_source::path.like(escape_like_starts_with(path_prefix_nocase)))
        }
        StringPredicateBorrowed::StartsNotWith(path_prefix_nocase) => statement
            .filter(media_source::path.not_like(escape_like_starts_with(path_prefix_nocase))),
        StringPredicateBorrowed::EndsWith(path_suffix_nocase) => {
            statement.filter(media_source::path.like(escape_like_ends_with(path_suffix_nocase)))
        }
        StringPredicateBorrowed::EndsNotWith(path_suffix_nocase) => {
            statement.filter(media_source::path.not_like(escape_like_ends_with(path_suffix_nocase)))
        }
        StringPredicateBorrowed::Contains(path_fragment_nocase) => {
            statement.filter(media_source::path.like(escape_like_contains(path_fragment_nocase)))
        }
        StringPredicateBorrowed::ContainsNot(path_fragment_nocase) => statement
            .filter(media_source::path.not_like(escape_like_contains(path_fragment_nocase))),
        StringPredicateBorrowed::Matches(path_fragment_nocase) => {
            statement.filter(media_source::path.like(escape_like_matches(path_fragment_nocase)))
        }
        StringPredicateBorrowed::MatchesNot(path_fragment_nocase) => {
            statement.filter(media_source::path.not_like(escape_like_matches(path_fragment_nocase)))
        }
        StringPredicateBorrowed::Prefix(path_prefix) => {
            let sql_prefix_filter = if path_prefix.contains('\'') {
                format!(
                    "substr(media_source.path,1,{})='{}'",
                    path_prefix.len(),
                    escape_single_quotes(path_prefix)
                )
            } else {
                format!(
                    "substr(media_source.path,1,{})='{}'",
                    path_prefix.len(),
                    path_prefix
                )
            };
            statement.filter(diesel::dsl::sql(&sql_prefix_filter))
        }
        StringPredicateBorrowed::Equals(path) => statement.filter(media_source::path.eq(path)),
        StringPredicateBorrowed::EqualsNot(path) => statement.filter(media_source::path.ne(path)),
    }
}
