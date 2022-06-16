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

use semval::Validate as _;

use aoide_core::track::{Track, TrackInvalidity};

use super::*;

pub mod find_duplicates;
pub mod purge;
pub mod replace;
pub mod resolve;
pub mod search;

#[cfg(feature = "media")]
pub mod find_unsynchronized;

#[cfg(feature = "media")]
pub mod import_and_replace;

#[cfg(feature = "media")]
pub mod vfs;

#[derive(Debug)]
pub struct ValidatedInput(Track);

pub fn validate_input(track: Track) -> InputResult<(ValidatedInput, Vec<TrackInvalidity>)> {
    // Many tracks are expected to be inconsistent and invalid to some
    // extent and we simply cannot reject all of them. The invalidities
    // are returned together with the validated input.
    let invalidities = track
        .validate()
        .map_err(|err| err.into_iter().collect())
        .err()
        .unwrap_or_default();
    Ok((ValidatedInput(track), invalidities))
}
