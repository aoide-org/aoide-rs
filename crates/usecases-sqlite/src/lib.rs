// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// rustflags
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
// rustflags (clippy)
#![warn(clippy::all)]
#![warn(clippy::explicit_deref_methods)]
#![warn(clippy::explicit_into_iter_loop)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::must_use_candidate)]
// rustdocflags
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use thiserror::Error;

use aoide_core::collection::EntityUid as CollectionUid;

use aoide_media::Error as MediaError;

use aoide_repo::prelude::RepoError;

use aoide_usecases as uc;

use aoide_repo_sqlite::{prelude::Connection as RepoConnection, DbConnection};

use aoide_storage_sqlite::Error as StorageError;

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
    Media(#[from] MediaError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    DatabaseMigration(anyhow::Error),

    #[error(transparent)]
    Repository(#[from] RepoError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Error::Storage(err.into())
    }
}

impl From<uc::Error> for Error {
    fn from(err: uc::Error) -> Self {
        use uc::Error::*;
        match err {
            Input(uc::InputError(err)) => Self::Input(err),
            Media(err) => Self::Media(err),
            Io(err) => Self::Io(err),
            Repository(err) => Self::Repository(err),
            Other(err) => Self::Other(err),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
