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

use aoide_core::util::url::BaseUrl;
use aoide_media::resolver::SourcePathResolver;

pub use aoide_repo::media::tracker::DirTrackingStatus;

pub mod import;
pub mod query_status;
pub mod relink;
pub mod scan;
pub mod untrack;

pub fn resolve_path_prefix_from_base_url(
    source_path_resolver: &impl SourcePathResolver,
    url_path_prefix: &BaseUrl,
) -> Result<SourcePath> {
    source_path_resolver
        .resolve_path_from_url(url_path_prefix)
        .map_err(|err| Error::Media(anyhow::format_err!("Invalid URL path prefix: {}", err).into()))
}
