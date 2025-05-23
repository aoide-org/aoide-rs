// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::result::Result as StdResult;

use thiserror::Error;

use aoide_repo::RepoError;

pub mod collection;
pub mod playlist;
pub mod track;

#[cfg(feature = "media-file")]
use aoide_media_file::Error as MediaFileError;

#[cfg(feature = "media-file")]
pub mod media;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct InputError(#[from] pub anyhow::Error);

pub type InputResult<T> = StdResult<T, InputError>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Input(#[from] InputError),

    #[cfg(feature = "media-file")]
    #[error(transparent)]
    MediaFile(#[from] MediaFileError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Repository(#[from] RepoError),

    #[error(transparent)]
    Other(anyhow::Error),
}

pub type Result<T> = StdResult<T, Error>;
