// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{
    FetchStatus, StartFindUntrackedFiles, StartImportFiles, StartScanDirectories,
    UntrackDirectories,
};
use crate::util::roundtrip::PendingToken;

#[derive(Debug, Clone)]
pub enum Task {
    Pending {
        token: PendingToken,
        task: PendingTask,
    },
}

#[derive(Debug, Clone)]
pub enum PendingTask {
    FetchProgress,
    FetchStatus(FetchStatus),
    StartScanDirectories(StartScanDirectories),
    StartImportFiles(StartImportFiles),
    StartFindUntrackedFiles(StartFindUntrackedFiles),
    UntrackDirectories(UntrackDirectories),
}

#[cfg(feature = "webapi-backend")]
mod webapi;
