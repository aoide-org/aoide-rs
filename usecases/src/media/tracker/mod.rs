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

use super::*;

use aoide_media::fs::local_file_path_from_url;

///////////////////////////////////////////////////////////////////////

pub mod hash;
pub mod import;
pub mod relink;

pub use aoide_repo::media::tracker::DirTrackingStatus;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Completion {
    Finished,
    Aborted,
}

pub fn root_dir_path_from_url(root_dir_url: &Url) -> Result<PathBuf> {
    if !root_dir_url.as_str().ends_with('/') {
        return Err(Error::Media(
            anyhow::format_err!("URL path does not end with a trailing slash").into(),
        ));
    }
    let root_dir_path = local_file_path_from_url(root_dir_url)?;
    if !root_dir_path.is_absolute() {
        return Err(Error::Media(
            anyhow::format_err!(
                "Root directory path is not absolute: {}",
                root_dir_path.display()
            )
            .into(),
        ));
    }
    Ok(root_dir_path)
}

pub fn path_prefix_from_url(url_path_prefix: &Url) -> Result<String> {
    if !url_path_prefix.as_str().ends_with('/') {
        return Err(Error::Media(
            anyhow::format_err!("URL path does not end with a trailing slash").into(),
        ));
    }
    Ok(url_path_prefix.to_string())
}
