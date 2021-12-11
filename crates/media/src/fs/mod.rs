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
    fs::File,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use url::Url;

use aoide_core::util::clock::DateTime;

use crate::{Error, IoError, Result};

pub mod digest;

pub fn file_path_from_url(url: &Url) -> Result<PathBuf> {
    if url.scheme() != "file" {
        return Err(Error::Io(IoError::new(
            ErrorKind::Other,
            anyhow!("Unsupported URL scheme '{}'", url.scheme()),
        )));
    }
    url.to_file_path().map_err(|()| {
        tracing::debug!(
            "Failed to convert URL '{}', into a local, absolute file path",
            url
        );
        Error::Io(IoError::new(
            ErrorKind::Other,
            anyhow!("Invalid or unsupported URL: {}", url),
        ))
    })
}

pub fn open_file_for_reading(file_path: impl AsRef<Path>) -> Result<Option<(PathBuf, File)>> {
    let canonical_path = file_path.as_ref().canonicalize()?;
    if canonical_path.is_dir() {
        return Ok(None);
    }
    let file = File::open(std::path::Path::new(&canonical_path))?;
    Ok(Some((canonical_path, file)))
}

pub fn file_last_modified_at(file: &File) -> Result<DateTime> {
    file.metadata()
        .map_err(Error::from)?
        .modified()
        .map(DateTime::from)
        .map_err(Error::from)
        .map(|last_modified_at| {
            if last_modified_at.timestamp_millis() > 0 {
                // Only consider time stamps strictly after the epoch origin
                // meaningful and valid
                last_modified_at
            } else {
                tracing::warn!(
                    "Using current time instead of invalid last modification time {}",
                    last_modified_at
                );
                DateTime::now_utc()
            }
        })
}
