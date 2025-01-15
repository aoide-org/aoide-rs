// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! File system utilities.

#[cfg(feature = "async-file-dialog")]
pub async fn choose_directory(
    dir_path: impl Into<Option<&std::path::Path>>,
) -> Option<aoide_core::util::fs::DirPath<'static>> {
    log::debug!("Open rfd::AsyncFileDialog");
    let mut file_dialog = rfd::AsyncFileDialog::new();
    if let Some(dir_path) = dir_path.into() {
        file_dialog = file_dialog.set_directory(dir_path);
    }
    let dir_handle = file_dialog.pick_folder().await;
    log::debug!("rfd::AsyncFileDialog closed");
    dir_handle.map(|file_handle| file_handle.path().to_path_buf().into())
}
