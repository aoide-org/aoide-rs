// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::anyhow;
use diesel::{
    expression::SqlLiteral,
    query_dsl::methods::{LimitDsl, OffsetDsl},
    sql_types,
};

use aoide_core_api::{
    Pagination,
    filter::{StringCompare, StringPredicate},
};
use aoide_repo::{RepoError, RepoResult};

pub(crate) mod clock;
pub(crate) mod entity;

fn pagination_to_limit_offset(pagination: &Pagination) -> (Option<i64>, Option<i64>) {
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

/// Applies the LIMIT/OFFSET of the given [`Pagination`] to a (boxed) query.
pub(crate) fn apply_pagination<Q>(mut query: Q, pagination: &Pagination) -> Q
where
    Q: LimitDsl<Output = Q> + OffsetDsl<Output = Q>,
{
    let (limit, offset) = pagination_to_limit_offset(pagination);
    if let Some(limit) = limit {
        query = query.limit(limit);
    }
    if let Some(offset) = offset {
        query = query.offset(offset);
    }
    query
}

/// Decodes an enum value from its integer representation in the database.
///
/// The `what` argument is used for the error message: `invalid {what} value: {value}`.
pub(crate) fn decode_repr<R, T>(
    value: i16,
    from_repr: impl FnOnce(R) -> Option<T>,
    what: &str,
) -> RepoResult<T>
where
    R: TryFrom<i16>,
{
    R::try_from(value)
        .ok()
        .and_then(from_repr)
        .ok_or_else(|| RepoError::Other(anyhow!("invalid {what} value: {value}")))
}

pub(crate) enum StringCmpOp {
    Equal(String),
    Prefix(String),
    Like { escaped: String },
}

impl StringCmpOp {
    pub(crate) fn new(cmp: StringCompare, val: &str) -> Self {
        match cmp {
            // Equal comparison without escape characters
            StringCompare::Equals => Self::Equal(val.to_owned()),
            StringCompare::Prefix => Self::Prefix(val.to_owned()),
            // Like comparisons with escaped wildcard character
            StringCompare::StartsWith => Self::Like {
                escaped: escape_like_starts_with(val),
            },
            StringCompare::EndsWith => Self::Like {
                escaped: escape_like_ends_with(val),
            },
            StringCompare::Contains => Self::Like {
                escaped: escape_like_contains(val),
            },
            StringCompare::Matches => Self::Like {
                escaped: escape_like_matches(val),
            },
        }
    }
}

/// Decomposes a [`StringPredicate`] into its value, comparison, and direction.
///
/// The direction is `true` for positive and `false` for negated predicates.
/// Returns `None` if the predicate matches all values and could be omitted.
pub(crate) fn decompose_string_predicate<'a>(
    p: &'a StringPredicate<'a>,
) -> Option<(&'a str, StringCompare, bool)> {
    use StringPredicate::{
        Contains, ContainsNot, EndsNotWith, EndsWith, Equals, EqualsNot, Matches, MatchesNot,
        Prefix, StartsNotWith, StartsWith,
    };
    let (s, cmp, dir) = match p {
        StartsWith(s) => {
            if s.is_empty() {
                // Include all
                return None;
            }
            (s, StringCompare::StartsWith, true)
        }
        StartsNotWith(s) => (s, StringCompare::StartsWith, false),
        EndsWith(s) => {
            if s.is_empty() {
                // Include all
                return None;
            }
            (s, StringCompare::EndsWith, true)
        }
        EndsNotWith(s) => (s, StringCompare::EndsWith, false),
        Contains(s) => {
            if s.is_empty() {
                // Include all
                return None;
            }
            (s, StringCompare::Contains, true)
        }
        ContainsNot(s) => (s, StringCompare::Contains, false),
        Matches(s) => (s, StringCompare::Matches, true),
        MatchesNot(s) => (s, StringCompare::Matches, false),
        Prefix(s) => {
            if s.is_empty() {
                // Include all
                return None;
            }
            (s, StringCompare::Prefix, true)
        }
        Equals(s) => (s, StringCompare::Equals, true),
        EqualsNot(s) => (s, StringCompare::Equals, false),
    };
    Some((s, cmp, dir))
}

pub(crate) const LIKE_ESCAPE_CHARACTER: char = '\\';
#[cfg(test)]
pub(crate) const LIKE_ESCAPE_CHARACTER_STR: &str = "\\";

pub(crate) const LIKE_WILDCARD_CHARACTER: char = '%';
pub(crate) const LIKE_WILDCARD_CHARACTER_STR: &str = "%";

pub(crate) const LIKE_PLACEHOLDER_CHARACTER: char = '_';
#[cfg(test)]
pub(crate) const LIKE_PLACEHOLDER_CHARACTER_STR: &str = "_";

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
    [&escape_like_matches(arg), LIKE_WILDCARD_CHARACTER_STR].concat()
}

pub(crate) fn escape_like_ends_with(arg: &str) -> String {
    [LIKE_WILDCARD_CHARACTER_STR, &escape_like_matches(arg)].concat()
}

pub(crate) fn escape_like_contains(arg: &str) -> String {
    [
        LIKE_WILDCARD_CHARACTER_STR,
        &escape_like_matches(arg),
        LIKE_WILDCARD_CHARACTER_STR,
    ]
    .concat()
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
            [LIKE_ESCAPE_CHARACTER_STR, LIKE_ESCAPE_CHARACTER_STR].concat(),
        );
        assert_eq!(
            LIKE_WILDCARD_CHARACTER_REPLACEMENT,
            [LIKE_ESCAPE_CHARACTER_STR, LIKE_WILDCARD_CHARACTER_STR].concat(),
        );
        assert_eq!(
            LIKE_PLACEHOLDER_CHARACTER_REPLACEMENT,
            [LIKE_ESCAPE_CHARACTER_STR, LIKE_PLACEHOLDER_CHARACTER_STR].concat(),
        );
    }
}
