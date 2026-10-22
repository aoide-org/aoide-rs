// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::{RunQueryDsl as _, dsl::sql, sql_types};

use super::like;
use crate::{
    DbConnection,
    tests::{TestResult, establish_connection},
};

fn eval(connection: &mut DbConnection, expr: &str) -> TestResult<Option<bool>> {
    diesel::select(sql::<sql_types::Nullable<sql_types::Bool>>(expr))
        .get_result(connection)
        .map_err(Into::into)
}

#[test]
fn like_wildcards() {
    assert!(like("", "", None));
    assert!(like("%", "", None));
    assert!(like("%", "abc", None));
    assert!(!like("_", "", None));
    assert!(like("_", "a", None));
    assert!(!like("_", "ab", None));
    assert!(like("a%c", "abbbc", None));
    assert!(like("a%c", "ac", None));
    assert!(!like("a%c", "acb", None));
    assert!(like("%b%", "abc", None));
    assert!(like("a_c", "abc", None));
    assert!(!like("a_c", "abbc", None));
    assert!(like("%a%a%", "xaxxax", None));
    assert!(!like("%a%a%", "xaxxx", None));
    assert!(like("%%_", "a", None));
    // `_` matches a single character, independent of its UTF-8 encoding.
    assert!(like("_", "é", None));
    assert!(like("___", "äöü", None));
}

#[test]
fn like_unicode_case_and_diacritics_folding() {
    assert!(like("abc", "ABC", None));
    assert!(like("Beyonce", "Beyoncé", None));
    assert!(like("beyoncé", "BEYONCE", None));
    assert!(like("a", "Ä", None));
    assert!(like("Ä", "a", None));
    assert!(like("ä", "Ä", None));
    assert!(like("aa", "Ää", None));
    assert!(like("%eeee%", "ÄäÖöÜüÉéÈè", None));
    assert!(like("%ÖÖ%", "ÄäÖöÜüÉéÈè", None));
    assert!(like("σ", "Σ", None));
    // Decomposed (NFD) and composed (NFC) representations are equivalent.
    assert!(like("e\u{301}", "\u{e9}", None));
    assert!(like("\u{e9}", "E\u{301}", None));
    assert!(!like("a", "b", None));
    assert!(!like("Beyonce", "Beyonc", None));
}

#[test]
fn like_escaped() {
    assert!(like("100\\%", "100%", Some('\\')));
    assert!(!like("100\\%", "1000", Some('\\')));
    assert!(like("a\\_c", "a_c", Some('\\')));
    assert!(!like("a\\_c", "abc", Some('\\')));
    assert!(like("a\\\\c", "a\\c", Some('\\')));
    assert!(like("%\\%%", "50% off", Some('\\')));
    assert!(!like("%\\%%", "50 off", Some('\\')));
    // Escaped characters are folded like any other literal character.
    assert!(like("\\É%", "eclair", Some('\\')));
    // Escape characters are matched verbatim, i.e. without folding.
    assert!(like("aÄ%Ä_", "A%_", Some('Ä')));
    assert!(!like("aÄ%Ä_", "A%x", Some('Ä')));
    assert!(!like("aä%", "A%", Some('Ä')));
    // Without an escape character the backslash is a literal character.
    assert!(like("a\\%", "a\\bc", None));
    assert!(!like("a\\%", "a%", None));
}

#[test]
fn sql_like_without_escape() -> TestResult<()> {
    let connection = &mut establish_connection()?;
    assert_eq!(Some(true), eval(connection, "'ABC' LIKE 'abc'")?);
    assert_eq!(Some(true), eval(connection, "'Beyoncé' LIKE 'Beyonce'")?);
    assert_eq!(Some(true), eval(connection, "'Beyonce' LIKE 'BEYONCÉ'")?);
    assert_eq!(Some(true), eval(connection, "'a' LIKE 'Ä'")?);
    assert_eq!(Some(true), eval(connection, "'Ä' LIKE 'a'")?);
    assert_eq!(Some(true), eval(connection, "'ÄäÖöÜüÉéÈè' LIKE '%aaoo%'")?);
    assert_eq!(Some(true), eval(connection, "'ÄäÖöÜüÉéÈè' LIKE 'ä_ö%è'")?);
    assert_eq!(Some(false), eval(connection, "'Beyoncé' LIKE 'Beyonc'")?);
    assert_eq!(
        Some(false),
        eval(connection, "'Beyoncé' NOT LIKE 'beyonce'")?
    );
    assert_eq!(Some(true), eval(connection, "'Beyoncé' NOT LIKE 'Beyonc'")?);
    // The backslash is a literal character if no escape character is given.
    assert_eq!(Some(false), eval(connection, "'100%' LIKE '100\\%'")?);
    assert_eq!(Some(true), eval(connection, "'100\\0' LIKE '100\\%'")?);
    // Direct invocation of the function with swapped arguments.
    assert_eq!(Some(true), eval(connection, "like('beyonce', 'Beyoncé')")?);
    // NULL propagation.
    assert_eq!(None, eval(connection, "NULL LIKE 'a'")?);
    assert_eq!(None, eval(connection, "'a' LIKE NULL")?);
    Ok(())
}

#[test]
fn sql_like_with_escape() -> TestResult<()> {
    let connection = &mut establish_connection()?;
    assert_eq!(
        Some(true),
        eval(connection, "'ABC' LIKE 'abc' ESCAPE '\\'")?
    );
    assert_eq!(
        Some(true),
        eval(connection, "'Beyoncé' LIKE 'Beyonce' ESCAPE '\\'")?
    );
    assert_eq!(Some(true), eval(connection, "'Ä' LIKE 'a' ESCAPE '\\'")?);
    assert_eq!(
        Some(true),
        eval(connection, "'100%' LIKE '100\\%' ESCAPE '\\'")?
    );
    assert_eq!(
        Some(false),
        eval(connection, "'1000' LIKE '100\\%' ESCAPE '\\'")?
    );
    assert_eq!(
        Some(true),
        eval(connection, "'Ä_É' LIKE 'a\\_e' ESCAPE '\\'")?
    );
    assert_eq!(
        Some(false),
        eval(connection, "'ÄxÉ' LIKE 'a\\_e' ESCAPE '\\'")?
    );
    assert_eq!(
        Some(true),
        eval(
            connection,
            "'50% Rabatt für Beyoncé' LIKE '%\\%%BEYONCE' ESCAPE '\\'"
        )?
    );
    assert_eq!(
        Some(true),
        eval(connection, "'ÄxÉ' NOT LIKE 'a\\_e' ESCAPE '\\'")?
    );
    // Direct invocation of the function with swapped arguments.
    assert_eq!(Some(true), eval(connection, "like('100#%', '100%', '#')")?);
    // NULL propagation.
    assert_eq!(None, eval(connection, "NULL LIKE 'a' ESCAPE '\\'")?);
    assert_eq!(None, eval(connection, "'a' LIKE NULL ESCAPE '\\'")?);
    assert_eq!(None, eval(connection, "'a' LIKE 'a' ESCAPE NULL")?);
    // Invalid escape expressions.
    assert_eq!(None, eval(connection, "'a' LIKE 'a' ESCAPE ''")?);
    assert_eq!(None, eval(connection, "'a' LIKE 'a' ESCAPE 'xy'")?);
    Ok(())
}
