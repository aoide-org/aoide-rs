// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::i64;

use diesel::expression::SqlLiteral;
use num_traits::ToPrimitive as _;

use crate::prelude::*;

pub(crate) mod clock;
pub(crate) mod entity;

pub(crate) fn apply_pagination<'db, ST, QS, DB>(
    source: diesel::query_builder::BoxedSelectStatement<'db, ST, QS, DB>,
    pagination: &Pagination,
) -> diesel::query_builder::BoxedSelectStatement<'db, ST, QS, DB>
where
    QS: diesel::query_source::QuerySource,
    DB: diesel::backend::Backend + diesel::sql_types::HasSqlType<ST> + 'db,
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

fn sql_column_substr_prefix(
    column: &str,
    prefix: &str,
    cmp: &str,
) -> SqlLiteral<diesel::sql_types::Bool> {
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

pub(crate) fn sql_column_substr_prefix_eq(
    column: &str,
    prefix: &str,
) -> SqlLiteral<diesel::sql_types::Bool> {
    sql_column_substr_prefix(column, prefix, "=")
}

pub(crate) fn sql_column_substr_prefix_ne(
    column: &str,
    prefix: &str,
) -> SqlLiteral<diesel::sql_types::Bool> {
    sql_column_substr_prefix(column, prefix, "<>")
}
