// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::track::find_unsynchronized::{Params, UnsynchronizedTrackEntity};

use aoide_usecases::track::find_unsynchronized as uc;

use super::*;

pub fn find_unsynchronized(
    connection: &mut SqliteConnection,
    collection_uid: &CollectionUid,
    params: Params,
    pagination: &Pagination,
) -> Result<Vec<UnsynchronizedTrackEntity>> {
    let mut repo = RepoConnection::new(connection);
    uc::find_unsynchronized_with_params(&mut repo, collection_uid, params, pagination)
        .map_err(Into::into)
}
