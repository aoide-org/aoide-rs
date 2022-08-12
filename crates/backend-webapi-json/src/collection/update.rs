// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_usecases_sqlite::collection::update as uc;

use super::*;

pub type QueryParams = EntityRevQueryParams;

pub type RequestBody = Collection;

pub type ResponseBody = Entity;

pub fn handle_request(
    connection: &mut DbConnection,
    uid: EntityUid,
    query_params: QueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let EntityRevQueryParams { rev } = query_params;
    let entity_header = _inner::EntityHeader { uid, rev };
    let modified_collection = request_body.try_into()?;
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::update(connection, entity_header, modified_collection).map_err(Into::into)
        })
        .map(Into::into)
}
