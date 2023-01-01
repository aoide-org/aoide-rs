// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::util::roundtrip::PendingToken;

use super::CollectionUid;

#[derive(Debug)]
pub enum Task {
    PurgeOrphaned {
        token: PendingToken,
        collection_uid: CollectionUid,
        params: aoide_core_api::media::source::purge_orphaned::Params,
    },
    PurgeUntracked {
        token: PendingToken,
        collection_uid: CollectionUid,
        params: aoide_core_api::media::source::purge_untracked::Params,
    },
}

#[cfg(feature = "webapi-backend")]
mod webapi;
