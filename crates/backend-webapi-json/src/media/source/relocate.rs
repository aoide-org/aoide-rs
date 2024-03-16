// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::media::content::ContentPath;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct RequestBody {
    old_path_prefix: String,
    new_path_prefix: String,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ResponseBody {
    replaced_count: usize,
}

pub fn handle_request(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let RequestBody {
        old_path_prefix,
        new_path_prefix,
    } = request_body;
    connection
        .transaction::<_, Error, _>(|connection| {
            aoide_usecases_sqlite::media::source::relocate::relocate(
                connection,
                collection_uid,
                &ContentPath::from(old_path_prefix),
                &ContentPath::from(new_path_prefix),
            )
            .map_err(Into::into)
        })
        .map(|replaced_count| ResponseBody { replaced_count })
}
