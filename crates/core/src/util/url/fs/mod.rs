// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    fs::{read_link, DirEntry},
    path::Path,
};

use ::url::Url;

#[allow(clippy::result_unit_err)]
pub fn url_from_path(path: &Path) -> Result<Url, ()> {
    if path.is_file() {
        Url::from_file_path(path)
    } else if path.is_dir() {
        Url::from_directory_path(path)
    } else {
        debug_assert!(
            false,
            "file type of path {} cannot be determined",
            path.display()
        );
        Err(())
    }
}

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
