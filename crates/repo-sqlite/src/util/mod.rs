// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::i64;

use aoide_core_api::Pagination;
use diesel::{expression::SqlLiteral, sql_types};

pub(crate) mod clock;
pub(crate) mod entity;

pub(crate) fn pagination_to_limit_offset(pagination: &Pagination) -> (Option<i64>, Option<i64>) {
    if !pagination.is_paginated() {
        return (None, None);
    }
    // SQLite: OFFSET can only be used in conjunction with LIMIT
    // according to the syntax diagram for the SELECT statement:
    // <https://www.sqlite.org/lang_select.html>
    let limit = if pagination.has_offset() || pagination.is_limited() {
        Some(pagination.mandatory_limit().try_into().unwrap_or(i64::MAX))
    } else {
        None
    };
    let offset = pagination
        .offset
        .map(|offset| offset.try_into().unwrap_or(i64::MAX));
    (limit, offset)
}

//FIXME: Figure types and trait bounds for a generic implementation of this function.
// pub(crate) fn apply_pagination<'db, Source>(
//     source: IntoBoxed<'db, Source, DbBackend>,
//     pagination: &Pagination,
// ) -> IntoBoxed<'db, Source, DbBackend>
// where
//     Source: BoxedDsl/LimitDsl/OffsetDsl???
// {
//     let (limit, offset) = pagination_to_limit_offset(pagination);
//     if let Some(limit) = limit {
//         target = target.limit(limit);
//     }
//     if let Some(offset) = offset {
//         target = target.offset(offset);
//     }
//     target
// }

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

fn sql_column_substr_prefix(column: &str, prefix: &str, cmp: &str) -> SqlLiteral<sql_types::Bool> {
    let prefix_len = prefix.len();
    if prefix.contains('\'') {
        let prefix_escaped = escape_single_quotes(prefix);
        diesel::dsl::sql::<sql_types::Bool>(&format!(
            "substr({column},1,{prefix_len}){cmp}'{prefix_escaped}'",
        ))
    } else {
        diesel::dsl::sql::<sql_types::Bool>(&format!(
            "substr({column},1,{prefix_len}){cmp}'{prefix}'",
        ))
    }
}

pub(crate) fn sql_column_substr_prefix_eq(
    column: &str,
    prefix: &str,
) -> SqlLiteral<sql_types::Bool> {
    sql_column_substr_prefix(column, prefix, "=")
}

pub(crate) fn sql_column_substr_prefix_ne(
    column: &str,
    prefix: &str,
) -> SqlLiteral<sql_types::Bool> {
    sql_column_substr_prefix(column, prefix, "<>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_character_and_replacements() {
        assert_eq!(
            LIKE_ESCAPE_CHARACTER_REPLACEMENT,
            format!("{LIKE_ESCAPE_CHARACTER}{LIKE_ESCAPE_CHARACTER}",)
        );
        assert_eq!(
            LIKE_WILDCARD_CHARACTER_REPLACEMENT,
            format!("{LIKE_ESCAPE_CHARACTER}{LIKE_WILDCARD_CHARACTER}",)
        );
        assert_eq!(
            LIKE_PLACEHOLDER_CHARACTER_REPLACEMENT,
            format!("{LIKE_ESCAPE_CHARACTER}{LIKE_PLACEHOLDER_CHARACTER}",)
        );
    }
}
