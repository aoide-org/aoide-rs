// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::time::{Duration, Instant};

use diesel::{
    Connection as _, RunQueryDsl as _, connection::SimpleConnection as _, dsl::sql, sql_types,
};

use super::like;
use crate::{
    DbConnection, initialize_connection,
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

const BENCH_ROW_COUNT: usize = 1_000_000;

const BENCH_RUN_COUNT: u32 = 3;

/// Optional path of a real database for running the search query
/// of the file collection app, e.g. `~/.mixxx/aoide.sqlite`.
const BENCH_DATABASE_ENV_VAR: &str = "AOIDE_BENCH_DATABASE";

#[derive(diesel::QueryableByName)]
struct BenchCount {
    #[diesel(sql_type = sql_types::BigInt)]
    count: i64,
}

/// Average execution time and result of a `SELECT count(*) AS count ...` query.
fn bench_count_query(connection: &mut DbConnection, query: &str) -> TestResult<(Duration, i64)> {
    // Warm up the page cache.
    diesel::sql_query(query).load::<BenchCount>(connection)?;
    let started = Instant::now();
    let mut count = 0;
    for _ in 0..BENCH_RUN_COUNT {
        count = diesel::sql_query(query).load::<BenchCount>(connection)?[0].count;
    }
    Ok((started.elapsed() / BENCH_RUN_COUNT, count))
}

fn bench_builtin_and_custom_like(
    mut connect: impl FnMut() -> TestResult<DbConnection>,
    query: &str,
) -> TestResult<()> {
    // A connection that has not been initialized uses the built-in LIKE.
    let (builtin_elapsed, builtin_count) = bench_count_query(&mut connect()?, query)?;
    let mut custom_connection = connect()?;
    initialize_connection(&mut custom_connection)?;
    let (custom_elapsed, custom_count) = bench_count_query(&mut custom_connection, query)?;
    println!("{query}");
    println!("built-in LIKE: {builtin_elapsed:>10.1?} (count = {builtin_count})");
    println!("custom LIKE:   {custom_elapsed:>10.1?} (count = {custom_count})");
    println!(
        "slowdown:      {:>10.1}x",
        custom_elapsed.as_secs_f64() / builtin_elapsed.as_secs_f64()
    );
    Ok(())
}

/// Baseline for the performance of the custom LIKE.
///
/// Run in release mode: `cargo test --release -p aoide-repo-sqlite -- --ignored --nocapture bench_sql_like`
#[test]
#[ignore = "benchmark"]
fn bench_sql_like() -> TestResult<()> {
    // Mostly ASCII text with some non-ASCII characters, like track titles and artist names.
    let words = [
        "Love",
        "Night",
        "Dancing",
        "Summer",
        "Daft",
        "Punk",
        "Club",
        "Mix",
        "Remix",
        "Edit",
        "Original",
        "Feat.",
        "Deep",
        "House",
        "Tribe",
        "Sound",
        "Heart",
        "City",
        "Dream",
        "Fire",
        "Light",
        "Soul",
        "Funk",
        "Dub",
        "Beyoncé",
        "Señorita",
        "Café",
        "Über",
        "Mötley",
        "Crüe",
        "Sigur",
        "Rós",
    ];
    let words_json = format!("[{}]", words.map(|word| format!("\"{word}\"")).join(","));
    let word_count = words.len();
    let create_table = format!(
        "CREATE TABLE bench_text (text TEXT NOT NULL);
        WITH RECURSIVE seq(i) AS (SELECT 0 UNION ALL SELECT i+1 FROM seq WHERE i < {max_row})
        INSERT INTO bench_text (text) SELECT
            (SELECT value FROM json_each('{words_json}') WHERE key = i % {word_count}) || ' ' ||
            (SELECT value FROM json_each('{words_json}') WHERE key = (i / {word_count}) % {word_count}) || ' ' ||
            (SELECT value FROM json_each('{words_json}') WHERE key = (i / {word_count} / {word_count}) % {word_count}) || ' ' ||
            i
        FROM seq;",
        max_row = BENCH_ROW_COUNT - 1,
    );
    // The in-memory database is shared by all connections while this one is open.
    let database_url = "file:bench_sql_like?mode=memory&cache=shared";
    let mut connection = DbConnection::establish(database_url)?;
    connection.batch_execute(&create_table)?;
    for pattern in ["%love%", "%beyonce%", "%xyz%"] {
        bench_builtin_and_custom_like(
            || DbConnection::establish(database_url).map_err(Into::into),
            &format!(
                "SELECT count(*) AS count FROM bench_text WHERE text LIKE '{pattern}' ESCAPE '\\'"
            ),
        )?;
    }

    let Ok(database) = std::env::var(BENCH_DATABASE_ENV_VAR) else {
        println!("Set {BENCH_DATABASE_ENV_VAR} to benchmark the search query on a real database");
        return Ok(());
    };
    // Search query of the file collection app for a single term.
    let query = "SELECT count(*) AS count FROM view_track_search WHERE
        row_id IN (SELECT track_id FROM track_title WHERE name LIKE '%love%' ESCAPE '\\')
        OR row_id IN (SELECT track_id FROM track_actor WHERE name LIKE '%love%' ESCAPE '\\')
        OR ifnull(publisher, '') LIKE '%love%' ESCAPE '\\'
        OR row_id IN (SELECT track_id FROM track_tag WHERE
            facet IN ('comm', 'cgrp', 'genre', 'mood', 'desc', 'xid', 'isrc')
            AND label LIKE '%love%' ESCAPE '\\')";
    bench_builtin_and_custom_like(
        || DbConnection::establish(&format!("file:{database}?mode=ro")).map_err(Into::into),
        query,
    )
}
