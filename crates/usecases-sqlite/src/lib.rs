// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// TODO: Remove temporary workaround.
// <https://github.com/rust-lang/rust-clippy/issues/11237>
#![allow(clippy::similar_names)]

use thiserror::Error;

use aoide_media_file::Error as MediaFileError;
use aoide_repo::RepoError;
use aoide_repo_sqlite::{Connection as RepoConnection, DbConnection};
use aoide_storage_sqlite::Error as StorageError;
use aoide_usecases as uc;

pub mod collection;
pub mod database;
pub mod media;
pub mod playlist;
pub mod track;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Input(anyhow::Error),

    #[error(transparent)]
    MediaFile(#[from] MediaFileError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    DatabaseMigration(anyhow::Error),

    #[error(transparent)]
    Repository(#[from] RepoError),

    #[error(transparent)]
    Other(anyhow::Error),
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Error::Storage(err.into())
    }
}

impl From<uc::Error> for Error {
    fn from(err: uc::Error) -> Self {
        use uc::Error as From;
        match err {
            From::Input(uc::InputError(err)) => Self::Input(err),
            From::MediaFile(err) => Self::MediaFile(err),
            From::Io(err) => Self::Io(err),
            From::Repository(err) => Self::Repository(err),
            From::Other(err) => Self::Other(err),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
