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

/// Filter by an URI predicate.
///
/// URIs are only unambiguous within a collection. Therefore
/// filtering is restricted to a single collection.
pub fn filter_by_uri_predicate<'db, DB>(
    collection_id: CollectionId,
    uri_predicate: StringPredicateBorrowed<'db>,
) -> BoxedSelectStatement<'db, BigInt, media_source::table, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    // Source URI filtering
    let statement = media_source::table
        .select(media_source::row_id)
        .filter(media_source::collection_id.eq(RowId::from(collection_id)))
        .into_boxed();
    match uri_predicate {
        StringPredicateBorrowed::StartsWith(uri_prefix_nocase) => {
            statement.filter(media_source::uri.like(escape_like_starts_with(uri_prefix_nocase)))
        }
        StringPredicateBorrowed::StartsNotWith(uri_prefix_nocase) => {
            statement.filter(media_source::uri.not_like(escape_like_starts_with(uri_prefix_nocase)))
        }
        StringPredicateBorrowed::EndsWith(uri_suffix_nocase) => {
            statement.filter(media_source::uri.like(escape_like_ends_with(uri_suffix_nocase)))
        }
        StringPredicateBorrowed::EndsNotWith(uri_suffix_nocase) => {
            statement.filter(media_source::uri.not_like(escape_like_ends_with(uri_suffix_nocase)))
        }
        StringPredicateBorrowed::Contains(uri_fragment_nocase) => {
            statement.filter(media_source::uri.like(escape_like_contains(uri_fragment_nocase)))
        }
        StringPredicateBorrowed::ContainsNot(uri_fragment_nocase) => {
            statement.filter(media_source::uri.not_like(escape_like_contains(uri_fragment_nocase)))
        }
        StringPredicateBorrowed::Matches(uri_fragment_nocase) => {
            statement.filter(media_source::uri.like(escape_like_matches(uri_fragment_nocase)))
        }
        StringPredicateBorrowed::MatchesNot(uri_fragment_nocase) => {
            statement.filter(media_source::uri.not_like(escape_like_matches(uri_fragment_nocase)))
        }
        StringPredicateBorrowed::Equals(uri) => statement.filter(media_source::uri.eq(uri)),
        StringPredicateBorrowed::EqualsNot(uri) => statement.filter(media_source::uri.ne(uri)),
    }
}
