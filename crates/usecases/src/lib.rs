// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// Opt-in for allowed-by-default lints (in alphabetical order)
// See also: <https://doc.rust-lang.org/rustc/lints>
#![warn(future_incompatible)]
#![warn(let_underscore)]
#![warn(missing_debug_implementations)]
//#![warn(missing_docs)] // TODO
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(unused)]
// Rustdoc lints
#![warn(rustdoc::broken_intra_doc_links)]
// Clippy lints
#![warn(clippy::pedantic)]
// Additional restrictions
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::self_named_module_files)]
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

use aoide_core::{prelude::*, CollectionUid};
use aoide_repo::prelude::*;
use thiserror::Error;

pub mod collection;
pub mod playlist;
pub mod track;

#[cfg(feature = "media-file")]
use aoide_media_file::Error as MediaFileError;

#[cfg(feature = "media-file")]
pub mod media;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct InputError(#[from] pub anyhow::Error);

pub type InputResult<T> = StdResult<T, InputError>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Input(#[from] InputError),

    #[cfg(feature = "media-file")]
    #[error(transparent)]
    MediaFile(#[from] MediaFileError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Repository(#[from] RepoError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = StdResult<T, Error>;
