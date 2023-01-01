// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

pub type RequestBody = aoide_core_api_json::media::source::purge_untracked::Params;

pub type ResponseBody = aoide_core_api_json::media::source::purge_untracked::Outcome;

#[allow(clippy::panic_in_result_fn)] // tracing::instrument
#[tracing::instrument(
    name = "Purging untracked media source",
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
            aoide_usecases_sqlite::media::source::purge_untracked::purge_untracked(
                connection,
                collection_uid,
                &params,
            )
            .map_err(Into::into)
        })
        .map(Into::into)
}
