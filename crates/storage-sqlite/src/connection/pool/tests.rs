// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::num::NonZeroU32;

use diesel::{
    QueryResult, RunQueryDsl as _, connection::SimpleConnection as _, dsl::sql, sql_types,
};

use super::{create_connection_pool, get_pooled_connection};
use crate::connection::Storage;

fn initialize_connection(connection: &mut diesel::SqliteConnection) -> QueryResult<()> {
    connection.batch_execute("PRAGMA recursive_triggers = 1; PRAGMA secure_delete = 1;")
}

fn select_pragma(connection: &mut diesel::SqliteConnection, name: &str) -> i64 {
    diesel::select(sql::<sql_types::BigInt>(&format!(
        "(SELECT {name} FROM pragma_{name}())"
    )))
    .get_result(connection)
    .unwrap()
}

#[test]
fn initialize_all_pooled_connections() {
    const MAX_SIZE: u32 = 3;
    let pool = create_connection_pool(
        &Storage::InMemory,
        NonZeroU32::new(MAX_SIZE).unwrap(),
        initialize_connection,
    )
    .unwrap();
    // Hold all connections concurrently to ensure that they are distinct.
    let mut connections = (0..MAX_SIZE)
        .map(|_| get_pooled_connection(&pool).unwrap())
        .collect::<Vec<_>>();
    for connection in &mut connections {
        assert_eq!(1, select_pragma(connection, "recursive_triggers"));
        assert_eq!(1, select_pragma(connection, "secure_delete"));
    }
}
