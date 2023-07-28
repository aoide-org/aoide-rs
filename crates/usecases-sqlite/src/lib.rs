// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
// Additional restrictions
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::mod_module_files)]
// Repetitions of module/type names occur frequently when using many
// modules for keeping the size of the source files handy. Often
// types have the same name as their parent module.
#![allow(clippy::module_name_repetitions)]
// Repeating the type name in `..Default::default()` expressions
// is not needed since the context is obvious.
#![allow(clippy::default_trait_access)]
// Using wildcard imports consciously is acceptable.
#![allow(clippy::wildcard_imports)]
// Importing all enum variants into a narrow, local scope is acceptable.
#![allow(clippy::enum_glob_use)]
// Both `collection_uid` and `collection_id` often appear within the same context.
#![allow(clippy::similar_names)]
// TODO: Add missing docs
#![allow(clippy::missing_errors_doc)]

use aoide_core::CollectionUid;
use aoide_media_file::Error as MediaFileError;
use aoide_repo::prelude::RepoError;
use aoide_repo_sqlite::{prelude::Connection as RepoConnection, DbConnection};
use aoide_storage_sqlite::Error as StorageError;
use aoide_usecases as uc;
use thiserror::Error;

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
            MediaFile(err) => Self::MediaFile(err),
            Io(err) => Self::Io(err),
            Repository(err) => Self::Repository(err),
            Other(err) => Self::Other(err),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
