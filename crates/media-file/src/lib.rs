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
// Clippy lints
#![warn(clippy::pedantic)]
// Additional restrictions
#![warn(clippy::clone_on_ref_ptr)]
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
// TODO: Review type casts
#![allow(clippy::cast_sign_loss)]
// TODO: Add missing docs
#![allow(clippy::missing_errors_doc)]

pub mod fmt;
pub mod fs;
pub mod io;
pub mod util;

use std::{io::Error as IoError, result::Result as StdResult};

use image::ImageError;
use lofty::LoftyError;
use mime::Mime;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown content type")]
    UnknownContentType(String),

    #[error("unsupported content type")]
    UnsupportedContentType(Mime),

    #[error("unsupported import options")]
    UnsupportedImportOptions,

    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Metadata(anyhow::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = StdResult<T, Error>;

impl From<mime::FromStrError> for Error {
    fn from(err: mime::FromStrError) -> Self {
        Self::UnknownContentType(err.to_string())
    }
}

impl From<LoftyError> for Error {
    fn from(err: LoftyError) -> Self {
        Self::Metadata(err.into())
    }
}

impl From<ImageError> for Error {
    fn from(err: ImageError) -> Self {
        match err {
            ImageError::IoError(err) => Self::Io(err),
            _ => Self::Metadata(err.into()),
        }
    }
}

pub mod prelude {
    pub use super::{Error, Result};
}
