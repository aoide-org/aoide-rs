// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_usecases_sqlite::playlist::create as uc;

use super::*;

pub type RequestBody = Playlist;

pub type ResponseBody = Entity;

pub fn handle_request(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::create(connection, collection_uid, request_body.into()).map_err(Into::into)
        })
        .map(Into::into)
}
