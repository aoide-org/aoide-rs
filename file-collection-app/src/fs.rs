// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

use aoide::util::fs::DirPath;

/// Open a file dialog to choose a directory path
///
/// Start with the given path if available.
///
/// Returns `Some` if a path has been chosen and `None` otherwise.
pub fn choose_directory_path<P>(
    rt: &tokio::runtime::Handle,
    dir_path: Option<&P>,
    on_dir_path_chosen: impl FnOnce(Option<DirPath<'static>>) + Send + 'static,
) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    let dir_path = dir_path.as_ref().map(AsRef::as_ref).map(PathBuf::from);
    rt.spawn(async move {
        let dir_path = aoide::desktop_app::fs::choose_directory(dir_path.as_deref()).await;
        on_dir_path_chosen(dir_path);
    });
    None
}
