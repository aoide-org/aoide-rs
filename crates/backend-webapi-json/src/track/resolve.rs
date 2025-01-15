// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_json::entity::EntityHeader;

use super::*;

mod uc {
    pub(super) use aoide_usecases_sqlite::track::resolve::*;
}

pub type RequestBody = Vec<String>;

pub type ResponseBody = Vec<(String, EntityHeader)>;

pub fn handle_request(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::resolve_by_media_source_content_paths(
                connection,
                collection_uid,
                request_body.into_iter().map(Into::into).collect(),
            )
            .map_err(Into::into)
        })
        .map(|v| {
            v.into_iter()
                .map(|(content_path, hdr)| (content_path.into(), hdr.into()))
                .collect()
        })
}
