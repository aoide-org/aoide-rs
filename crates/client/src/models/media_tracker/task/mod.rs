// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{prelude::*, util::roundtrip::PendingToken};

#[derive(Debug)]
pub enum Task {
    FetchProgress {
        token: PendingToken,
    },
    FetchStatus {
        token: PendingToken,
        collection_uid: CollectionUid,
        params: aoide_core_api::media::tracker::query_status::Params,
    },
    StartScanDirectories {
        token: PendingToken,
        collection_uid: CollectionUid,
        params: aoide_core_api::media::tracker::scan_directories::Params,
    },
    StartImportFiles {
        token: PendingToken,
        collection_uid: CollectionUid,
        params: aoide_core_api::media::tracker::import_files::Params,
    },
    StartFindUntrackedFiles {
        token: PendingToken,
        collection_uid: CollectionUid,
        params: aoide_core_api::media::tracker::find_untracked_files::Params,
    },
    UntrackDirectories {
        token: PendingToken,
        collection_uid: CollectionUid,
        params: aoide_core_api::media::tracker::untrack_directories::Params,
    },
}

#[cfg(feature = "webapi-backend")]
mod webapi;
