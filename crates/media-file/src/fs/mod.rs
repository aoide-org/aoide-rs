// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fs::File,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use url::Url;

use crate::{Error, IoError, Result};

pub mod digest;
pub mod visit;

pub fn file_path_from_url(url: &Url) -> Result<PathBuf> {
    let url_scheme = url.scheme();
    if url_scheme != "file" {
        return Err(Error::Io(IoError::new(
            ErrorKind::Other,
            anyhow!("Unsupported URL scheme '{url_scheme}'"),
        )));
    }
    url.to_file_path().map_err(|()| {
        log::debug!("Failed to convert URL '{url}', into a local, absolute file path");
        Error::Io(IoError::new(
            ErrorKind::Other,
            anyhow!("Invalid or unsupported URL: {url}"),
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
