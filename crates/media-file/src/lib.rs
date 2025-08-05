// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod fmt;
pub mod fs;
pub mod io;
pub mod util;

use std::{io::Error as IoError, result::Result as StdResult};

use image::ImageError;
use lofty::error::LoftyError;
use mime::Mime;
use thiserror::Error;

use aoide_core::media::artwork::ArtworkImageError;

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
    // TODO: Remove implicit conversion from anyhow::Error.
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

impl From<ArtworkImageError> for crate::Error {
    fn from(err: ArtworkImageError) -> crate::Error {
        match err {
            ArtworkImageError::UnsupportedFormat(image_format) => Self::Metadata(anyhow::anyhow!(
                "unsupported artwork image format: {image_format:?}"
            )),
            ArtworkImageError::Image(err) => err.into(),
            ArtworkImageError::Other(err) => Self::Metadata(err),
        }
    }
}

pub mod prelude {
    pub use super::{Error, Result};
}
