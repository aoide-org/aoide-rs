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

use aoide_repo::media::tracker::Repo as _;

pub mod hash;
pub mod import;
pub mod query_status;
pub mod relink;
pub mod untrack;

pub use aoide_repo::media::tracker::DirTrackingStatus;

pub use aoide_usecases::media::tracker::*;

pub fn path_prefix_from_url(url_path_prefix: &Url) -> Result<String> {
    if !url_path_prefix.as_str().ends_with('/') {
        return Err(Error::Media(
            anyhow::format_err!("URL path does not end with a trailing slash").into(),
        ));
    }
    Ok(url_path_prefix.to_string())
}
