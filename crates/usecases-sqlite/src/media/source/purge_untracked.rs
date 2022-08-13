// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::media::source::purge_untracked::{Outcome, Params};

use super::*;

mod uc {
    pub(super) use aoide_usecases::media::source::purge_untracked::purge_untracked;
}

pub fn purge_untracked(
    connection: &mut SqliteConnection,
    collection_uid: &CollectionUid,
    params: &Params,
) -> Result<Outcome> {
    let mut repo = RepoConnection::new(connection);
    uc::purge_untracked(&mut repo, collection_uid, params).map_err(Into::into)
}
