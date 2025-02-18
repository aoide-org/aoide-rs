// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{CollectionUid, media::content::ContentPath};
use aoide_core_api::media::tracker::count_sources_in_directories::Params;
use aoide_repo_sqlite::DbConnection;

use crate::{RepoConnection, Result};

mod uc {
    pub(super) use aoide_usecases::media::tracker::count_sources_in_directories::*;
}

pub fn count_sources_in_directories(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Vec<(ContentPath<'static>, usize)>> {
    let mut repo = RepoConnection::new(connection);
    uc::count_sources_in_directories(&mut repo, collection_uid, params).map_err(Into::into)
}
