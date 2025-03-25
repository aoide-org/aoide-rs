// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

mod uc {
    pub(super) use aoide_usecases_sqlite::media::tracker::untrack_directories::*;
}

pub type RequestBody = aoide_core_api_json::media::tracker::untrack_directories::Params;

pub type ResponseBody = aoide_core_api_json::media::tracker::untrack_directories::Outcome;

#[tracing::instrument(
    name = "Untracking media sources",
    skip(
        connection,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let params = request_body
        .try_into()
        .map_err(Into::into)
        .map_err(Error::BadRequest)?;
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::untrack_directories(connection, collection_uid, &params).map_err(Into::into)
        })
        .map(Into::into)
}
