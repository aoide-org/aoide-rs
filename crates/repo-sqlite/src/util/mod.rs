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

use std::i64;

use diesel::expression::SqlLiteral;
use num_traits::ToPrimitive as _;

use crate::prelude::*;

pub(crate) mod clock;
pub(crate) mod entity;

pub(crate) fn apply_pagination<'a, ST, QS, DB>(
    source: diesel::query_builder::BoxedSelectStatement<'a, ST, QS, DB>,
    pagination: &Pagination,
) -> diesel::query_builder::BoxedSelectStatement<'a, ST, QS, DB>
where
    QS: diesel::query_source::QuerySource,
    DB: diesel::backend::Backend + diesel::sql_types::HasSqlType<ST> + 'a,
{
    if !pagination.is_paginated() {
        return source;
    }
    let mut target = source;
    // TODO: Verify that this restriction still applies!
    // SQLite: OFFSET can only be used in conjunction with LIMIT
    if pagination.has_offset() || pagination.is_limited() {
        let limit = pagination.mandatory_limit().to_i64().unwrap_or(i64::MAX);
        target = target.limit(limit);
    }
    if let Some(offset) = pagination.offset {
        let offset = offset.to_i64().unwrap_or(i64::MAX);
        target = target.offset(offset);
    }
    target
}

pub(crate) enum StringCmpOp {
    Equal(String),
    Prefix(String, usize),
    Like(String),
}

pub(crate) const LIKE_ESCAPE_CHARACTER: char = '\\';

pub(crate) const LIKE_WILDCARD_CHARACTER: char = '%';
pub(crate) const LIKE_PLACEHOLDER_CHARACTER: char = '_';

const LIKE_ESCAPE_CHARACTER_REPLACEMENT: &str = "\\\\"; // LIKE_ESCAPE_CHARACTER + LIKE_ESCAPE_CHARACTER

const LIKE_WILDCARD_CHARACTER_REPLACEMENT: &str = "\\%"; // LIKE_ESCAPE_CHARACTER + LIKE_WILDCARD_CHARACTER
const LIKE_PLACEHOLDER_CHARACTER_REPLACEMENT: &str = "\\_"; // LIKE_ESCAPE_CHARACTER + LIKE_PLACEHOLDER_CHARACTER

pub(crate) fn escape_like_matches(arg: &str) -> String {
    // The order if replacements matters!
    arg.replace(LIKE_ESCAPE_CHARACTER, LIKE_ESCAPE_CHARACTER_REPLACEMENT)
        .replace(LIKE_WILDCARD_CHARACTER, LIKE_WILDCARD_CHARACTER_REPLACEMENT)
        .replace(
            LIKE_PLACEHOLDER_CHARACTER,
            LIKE_PLACEHOLDER_CHARACTER_REPLACEMENT,
        )
}

pub(crate) fn escape_single_quotes(arg: &str) -> String {
    arg.replace('\'', "''")
}

pub(crate) fn escape_like_starts_with(arg: &str) -> String {
    format!("{}{LIKE_WILDCARD_CHARACTER}", escape_like_matches(arg))
}

pub(crate) fn escape_like_ends_with(arg: &str) -> String {
    format!("{LIKE_WILDCARD_CHARACTER}{}", escape_like_matches(arg))
}

pub(crate) fn escape_like_contains(arg: &str) -> String {
    format!(
        "{LIKE_WILDCARD_CHARACTER}{}{LIKE_WILDCARD_CHARACTER}",
        escape_like_matches(arg),
    )
}

fn sql_column_substr_prefix<ST>(column: &str, prefix: &str, cmp: &str) -> SqlLiteral<ST> {
    let prefix_len = prefix.len();
    if prefix.contains('\'') {
        let prefix_escaped = escape_single_quotes(prefix);
        diesel::dsl::sql(&format!(
            "substr({column},1,{prefix_len}){cmp}'{prefix_escaped}'",
        ))
    } else {
        diesel::dsl::sql(&format!("substr({column},1,{prefix_len}){cmp}'{prefix}'",))
    }
}

pub(crate) fn sql_column_substr_prefix_eq<ST>(column: &str, prefix: &str) -> SqlLiteral<ST> {
    sql_column_substr_prefix(column, prefix, "=")
}

pub(crate) fn sql_column_substr_prefix_ne<ST>(column: &str, prefix: &str) -> SqlLiteral<ST> {
    sql_column_substr_prefix(column, prefix, "<>")
}
