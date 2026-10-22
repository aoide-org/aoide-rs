// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unicode-aware replacement for the built-in `LIKE` operator of SQLite.
//!
//! The built-in `LIKE` operator only folds the case of ASCII characters.
//! The replacement folds the case of all Unicode characters and ignores
//! diacritics, i.e. "Beyonce" matches "Beyoncé" and "Ä" matches "a".
//!
//! SQLite implements `X LIKE Y` as `like(Y, X)` and `X LIKE Y ESCAPE Z`
//! as `like(Y, X, Z)`. Both functions are overloaded separately.
//!
//! <https://www.sqlite.org/lang_corefunc.html#like>

use diesel::{
    QueryResult, define_sql_function,
    sql_types::{Bool, Nullable, Text},
};
use unicode_normalization::{UnicodeNormalization as _, char::is_combining_mark};

use crate::DbConnection;

define_sql_function! {
    #[sql_name = "like"]
    fn like2(pattern: Nullable<Text>, text: Nullable<Text>) -> Nullable<Bool>;
}

define_sql_function! {
    #[sql_name = "like"]
    fn like3(pattern: Nullable<Text>, text: Nullable<Text>, escape: Nullable<Text>) -> Nullable<Bool>;
}

/// Override the built-in `LIKE` operator.
///
/// Must be invoked for each connection.
pub(crate) fn register_functions(connection: &mut DbConnection) -> QueryResult<()> {
    like2_utils::register_impl(
        connection,
        |pattern: Option<String>, text: Option<String>| {
            Some(like(pattern.as_deref()?, text.as_deref()?, None))
        },
    )?;
    like3_utils::register_impl(
        connection,
        |pattern: Option<String>, text: Option<String>, escape: Option<String>| {
            let escape = escape?;
            let mut escape_chars = escape.chars();
            let (Some(escape_char), None) = (escape_chars.next(), escape_chars.next()) else {
                // The built-in function fails if the escape expression is not
                // a single character. Errors cannot be reported and panicking
                // is not an option, i.e. the result is undefined (NULL).
                return None;
            };
            Some(like(
                pattern.as_deref()?,
                text.as_deref()?,
                Some(escape_char),
            ))
        },
    )?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    /// A single (folded) character.
    Char(char),
    /// `_`: Any single character.
    AnyChar,
    /// `%`: Any sequence of zero or more characters.
    AnySeq,
}

fn fold_chars(s: &str) -> impl Iterator<Item = char> + '_ {
    s.nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .filter(|c| !is_combining_mark(*c))
}

fn tokenize_pattern(pattern: &str, escape: Option<char>) -> Vec<Token> {
    let mut tokens = Vec::with_capacity(pattern.len());
    // Consecutive literal characters are folded together to preserve
    // the decomposition of base characters and combining marks.
    let mut literal = String::new();
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        let wildcard = if Some(c) == escape {
            if let Some(escaped) = chars.next() {
                literal.push(escaped);
            }
            // A trailing escape character is ignored.
            None
        } else {
            match c {
                '%' => Some(Token::AnySeq),
                '_' => Some(Token::AnyChar),
                c => {
                    literal.push(c);
                    None
                }
            }
        };
        if let Some(wildcard) = wildcard {
            tokens.extend(fold_chars(&literal).map(Token::Char));
            literal.clear();
            tokens.push(wildcard);
        }
    }
    tokens.extend(fold_chars(&literal).map(Token::Char));
    tokens
}

fn like(pattern: &str, text: &str, escape: Option<char>) -> bool {
    let pattern = tokenize_pattern(pattern, escape);
    let text = fold_chars(text).collect::<Vec<_>>();
    // Iterative wildcard matching with backtracking to the last `%`.
    let mut pattern_pos = 0;
    let mut text_pos = 0;
    let mut backtrack = None;
    while text_pos < text.len() {
        match pattern.get(pattern_pos) {
            Some(Token::AnySeq) => {
                pattern_pos += 1;
                backtrack = Some((pattern_pos, text_pos));
                continue;
            }
            Some(Token::AnyChar) => {
                pattern_pos += 1;
                text_pos += 1;
                continue;
            }
            Some(Token::Char(c)) if *c == text[text_pos] => {
                pattern_pos += 1;
                text_pos += 1;
                continue;
            }
            _ => (),
        }
        let Some((backtrack_pattern_pos, backtrack_text_pos)) = backtrack else {
            return false;
        };
        // Let the last `%` consume one more character.
        pattern_pos = backtrack_pattern_pos;
        text_pos = backtrack_text_pos + 1;
        backtrack = Some((backtrack_pattern_pos, text_pos));
    }
    pattern[pattern_pos..]
        .iter()
        .all(|token| *token == Token::AnySeq)
}

#[cfg(test)]
mod tests;
