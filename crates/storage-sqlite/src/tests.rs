// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::{
    Connection as _, SqliteConnection,
    connection::SimpleConnection as _,
    sqlite::{
        AutoVacuumMode,
        pragma::{AutoVacuum, FreelistCount, SchemaPragma as _},
    },
};

use super::{VacuumMode, vacuum_database};

/// Create a database with many pages on the freelist.
fn establish_fragmented_connection() -> SqliteConnection {
    let mut connection = SqliteConnection::establish(":memory:").unwrap();
    AutoVacuum::set(AutoVacuumMode::Incremental, None, &mut connection).unwrap();
    connection
        .batch_execute(
            "CREATE TABLE t (data BLOB);
             WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 100)
             INSERT INTO t SELECT randomblob(4096) FROM n;
             DELETE FROM t;",
        )
        .unwrap();
    assert!(FreelistCount::get::<i64>(None, &mut connection).unwrap() > 1);
    connection
}

#[test]
fn incremental_vacuum_releases_whole_freelist() {
    let mut connection = establish_fragmented_connection();
    vacuum_database(&mut connection, VacuumMode::Incremental).unwrap();
    assert_eq!(0, FreelistCount::get::<i64>(None, &mut connection).unwrap());
}

#[test]
fn full_vacuum_releases_whole_freelist() {
    let mut connection = establish_fragmented_connection();
    vacuum_database(&mut connection, VacuumMode::Full).unwrap();
    assert_eq!(0, FreelistCount::get::<i64>(None, &mut connection).unwrap());
}
