// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_usecases_sqlite::playlist as uc;

use super::*;

pub type RequestBody = Playlist;

pub type ResponseBody = Entity;

#[allow(clippy::needless_pass_by_value)] // consume arguments
pub fn handle_request(
    connection: &mut DbConnection,
    uid: EntityUid,
    query_params: EntityRevQueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let EntityRevQueryParams { rev } = query_params;
    let entity_header = _core::EntityHeader { uid, rev };
    let modified_playlist = request_body.into();
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::update(connection, entity_header, modified_playlist).map_err(Into::into)
        })
        .map(Into::into)
}
