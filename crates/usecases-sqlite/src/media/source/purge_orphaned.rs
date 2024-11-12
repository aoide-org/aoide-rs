// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::CollectionUid;
use aoide_core_api::media::source::purge_orphaned::{Outcome, Params};
use aoide_repo_sqlite::DbConnection;

use crate::{RepoConnection, Result};

mod uc {
    pub(super) use aoide_usecases::media::source::purge_orphaned::purge_orphaned;
}

pub fn purge_orphaned(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Outcome> {
    let mut repo = RepoConnection::new(connection);
    uc::purge_orphaned(&mut repo, collection_uid, params).map_err(Into::into)
}
