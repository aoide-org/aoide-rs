// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use diesel::prelude::*;
use thiserror::Error;

use aoide_core::collection::EntityUid as CollectionUid;

use aoide_media::Error as MediaError;

use aoide_repo::prelude::RepoError;

use aoide_usecases as uc;

use aoide_repo_sqlite::prelude::Connection as RepoConnection;

use aoide_storage_sqlite::Error as StorageError;

#[macro_use]
extern crate diesel_migrations;

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
    DatabaseMigration(#[from] diesel_migrations::RunMigrationsError),

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
