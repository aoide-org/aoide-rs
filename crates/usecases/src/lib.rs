// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![warn(clippy::clone_on_ref_ptr)]
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

use std::result::Result as StdResult;

use thiserror::Error;

use aoide_core::{collection::EntityUid as CollectionUid, prelude::*};
use aoide_repo::prelude::*;

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
