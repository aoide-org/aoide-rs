// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// TODO: Remove temporary workaround.
// <https://github.com/rust-lang/rust-clippy/issues/11237>
#![allow(clippy::wildcard_imports)]

pub mod fmt;
pub mod fs;
pub mod io;
pub mod util;

use std::{io::Error as IoError, result::Result as StdResult};

use image::ImageError;
use lofty::LoftyError;
use mime::Mime;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown content type")]
    UnknownContentType(String),

    #[error("unsupported content type")]
    UnsupportedContentType(Mime),

    #[error("unsupported import options")]
    UnsupportedImportOptions,

    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Metadata(anyhow::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = StdResult<T, Error>;

impl From<mime::FromStrError> for Error {
    fn from(err: mime::FromStrError) -> Self {
        Self::UnknownContentType(err.to_string())
    }
}

impl From<LoftyError> for Error {
    fn from(err: LoftyError) -> Self {
        Self::Metadata(err.into())
    }
}

impl From<ImageError> for Error {
    fn from(err: ImageError) -> Self {
        match err {
            ImageError::IoError(err) => Self::Io(err),
            _ => Self::Metadata(err.into()),
        }
    }
}

pub mod prelude {
    pub use super::{Error, Result};
}
