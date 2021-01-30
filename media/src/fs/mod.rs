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

///////////////////////////////////////////////////////////////////////

pub mod dir_digest;

use super::{Error, IoError, Result};

use anyhow::anyhow;
use std::{fs::File, io::ErrorKind};
use url::Url;

pub use mime::Mime;

pub fn open_local_file_url_for_reading(url: &Url) -> Result<File> {
    log::debug!("Opening local file URL '{}' for reading", url);
    if url.scheme() != "file" {
        return Err(Error::Io(IoError::new(
            ErrorKind::Other,
            anyhow!("Unsupported URL scheme '{}'", url.scheme()),
        )));
    }
    if let Ok(file_path) = url.to_file_path() {
        log::debug!("Importing track from local file {:?}", file_path);
        Ok(File::open(std::path::Path::new(&file_path))?)
    } else {
        log::debug!(
            "Failed to convert URL '{}', into a local, absolute file path",
            url
        );
        Err(Error::Io(IoError::new(
            ErrorKind::Other,
            anyhow!("Invalid or unsupported URL: {}", url),
        )))
    }
}
