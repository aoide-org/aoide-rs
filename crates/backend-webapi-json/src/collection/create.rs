// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_usecases_sqlite::collection::create as uc;

use super::*;

pub type RequestBody = Collection;

pub type ResponseBody = Entity;

pub fn handle_request(
    connection: &SqliteConnection,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let created_collection = request_body.try_into()?;
    connection
        .transaction::<_, Error, _>(|| {
            uc::create(connection, created_collection).map_err(Into::into)
        })
        .map(Into::into)
}
