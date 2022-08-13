// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

mod uc {
    pub(super) use aoide_usecases_sqlite::media::tracker::query_status::*;
}

pub type RequestBody = aoide_core_api_json::media::tracker::query_status::Params;
pub type ResponseBody = aoide_core_api_json::media::tracker::Status;

pub fn handle_request(
    connection: &mut SqliteConnection,
    collection_uid: &CollectionUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let params = request_body
        .try_into()
        .map_err(Into::into)
        .map_err(Error::BadRequest)?;
    //FIXME: Add transactions after upgrading to diesel v2.0
    //connection.transaction::<_, Error, _>(|connection| {
    uc::query_status(connection, collection_uid, &params).map_err(Into::into)
        //})
        .map(Into::into)
}
