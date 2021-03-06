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

use super::*;

use aoide_media::Error as MediaError;
use aoide_repo::prelude::RepoError;
use aoide_usecases as uc;

use std::result::Result as StdResult;
use thiserror::Error;

///////////////////////////////////////////////////////////////////////

pub mod collections;
pub mod database;
pub mod media;
pub mod playlists;
pub mod tracks;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Media(#[from] MediaError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Database(#[from] diesel::result::Error),

    #[error(transparent)]
    DatabaseMigration(#[from] diesel_migrations::RunMigrationsError),

    #[error(transparent)]
    DatabaseConnection(#[from] r2d2::Error),

    #[error(transparent)]
    Repository(#[from] RepoError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl<E> From<DieselTransactionError<E>> for Error
where
    E: Into<Error>,
{
    fn from(err: DieselTransactionError<E>) -> Self {
        err.into_inner().into()
    }
}

impl From<uc::Error> for Error {
    fn from(err: uc::Error) -> Self {
        use uc::Error::*;
        match err {
            Media(err) => Self::Media(err),
            Io(err) => Self::Io(err),
            Repository(err) => Self::Repository(err),
            Other(err) => Self::Other(err),
        }
    }
}

pub type Result<T> = StdResult<T, Error>;
