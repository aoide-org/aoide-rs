// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_usecases_sqlite::collection::create as uc;

use super::*;

pub type RequestBody = Collection;

pub type ResponseBody = Entity;

pub fn handle_request(
    connection: &mut SqliteConnection,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let created_collection = request_body.try_into()?;
    //FIXME: Add transactions after upgrading to diesel v2.0
    //connection.transaction::<_, Error, _>(|connection| {
    uc::create(connection, created_collection).map_err(Into::into)
        //})
        .map(Into::into)
}
