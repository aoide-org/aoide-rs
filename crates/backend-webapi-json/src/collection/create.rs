// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_usecases_sqlite::collection as uc;

use super::*;

pub type RequestBody = Collection;

pub type ResponseBody = Entity;

pub fn handle_request(
    connection: &mut DbConnection,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let created_collection = request_body.try_into().map_err(Error::Other)?;
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::create(connection, created_collection).map_err(Into::into)
        })
        .map(Into::into)
}
