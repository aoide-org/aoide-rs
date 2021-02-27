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

use std::result::Result as StdResult;
use thiserror::Error;

///////////////////////////////////////////////////////////////////////

pub mod collections;
pub mod database;
pub mod media;
pub mod playlists;
pub mod relink_collected_track;
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

impl From<DieselRepoError> for Error {
    fn from(err: DieselRepoError) -> Self {
        Self::Repository(err.into())
    }
}

pub type Result<T> = StdResult<T, Error>;
