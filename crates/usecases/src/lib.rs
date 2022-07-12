// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::explicit_deref_methods)]
#![warn(clippy::explicit_into_iter_loop)]
#![warn(clippy::explicit_iter_loop)]
#![warn(clippy::must_use_candidate)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use aoide_core::collection::EntityUid as CollectionUid;
use aoide_repo::prelude::*;

use std::result::Result as StdResult;
use thiserror::Error;

pub mod collection;
pub mod playlist;
pub mod track;

#[cfg(feature = "media")]
use aoide_media::Error as MediaError;

#[cfg(feature = "media")]
pub mod media;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct InputError(#[from] pub anyhow::Error);

pub type InputResult<T> = StdResult<T, InputError>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Input(#[from] InputError),

    #[cfg(feature = "media")]
    #[error(transparent)]
    Media(#[from] MediaError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Repository(#[from] RepoError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = StdResult<T, Error>;
