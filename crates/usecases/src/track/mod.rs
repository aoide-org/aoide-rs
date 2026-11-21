// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use semval::prelude::*;

use aoide_core::{Track, track::TrackInvalidity};

pub mod find_duplicates;
pub mod purge;
pub mod replace;
pub mod resolve;
pub mod search;

#[cfg(not(target_family = "wasm"))]
pub mod find_unsynchronized;

#[cfg(all(feature = "media-file", not(target_family = "wasm")))]
pub mod import_and_replace;

#[cfg(all(feature = "media-file", not(target_family = "wasm")))]
pub mod update;

#[cfg(not(target_family = "wasm"))]
pub mod vfs;

#[derive(Debug)]
pub struct ValidatedInput(Track);

/// Validate a track without rejecting it.
///
/// Many tracks are expected to be inconsistent and invalid to some
/// extent and we simply cannot reject all of them. The invalidities
/// are returned together with the validated input.
#[must_use]
pub fn validate_input(track: Track) -> (ValidatedInput, Vec<TrackInvalidity>) {
    let invalidities = track
        .validate()
        .map_err(|err| err.into_iter().collect())
        .err()
        .unwrap_or_default();
    (ValidatedInput(track), invalidities)
}
