// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::CollectionUid;
use aoide_core_api::media::tracker::{query_status::Params, Status};
use aoide_repo_sqlite::DbConnection;

use crate::{RepoConnection, Result};

mod uc {
    pub(super) use aoide_usecases::media::tracker::query_status::*;
}

pub fn query_status(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Status> {
    let mut repo = RepoConnection::new(connection);
    uc::query_status(&mut repo, collection_uid, params).map_err(Into::into)
}
