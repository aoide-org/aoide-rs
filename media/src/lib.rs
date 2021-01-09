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

///////////////////////////////////////////////////////////////////////

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

use aoide_core::track::Track;

use std::{io::{Read, Error as IoError}, result::Result as StdResult};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = StdResult<T, Error>;

pub trait ReadTrack {
    fn read_track(read: &dyn Read) -> Result<Track>;
}

#[cfg(feature = "feature-flac")]
pub mod flac;

#[cfg(feature = "feature-mp3")]
pub mod mp3;

#[cfg(feature = "feature-mp4")]
pub mod mp4;

#[cfg(feature = "feature-ogg")]
pub mod ogg;

#[cfg(feature = "feature-wav")]
pub mod wav;
