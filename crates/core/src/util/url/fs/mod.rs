// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fs::{DirEntry, read_link},
    path::Path,
};

use ::url::Url;

#[expect(clippy::result_unit_err)]
pub fn url_from_path(path: &Path) -> Result<Url, ()> {
    if path.is_file() {
        Url::from_file_path(path)
    } else if path.is_dir() {
        Url::from_directory_path(path)
    } else {
        debug_assert!(
            false,
            "file type of path {path} cannot be determined",
            path = path.display()
        );
        Err(())
    }
}

#[expect(clippy::missing_panics_doc)] // Never panics
pub fn url_from_dir_entry(dir_entry: &DirEntry) -> std::io::Result<Url> {
    let file_type = dir_entry.file_type()?;
    let url =
        if file_type.is_dir() || file_type.is_symlink() && read_link(dir_entry.path())?.is_dir() {
            Url::from_directory_path(dir_entry.path())
        } else {
            Url::from_file_path(dir_entry.path())
        }
        .expect("URL");
    Ok(url)
}
