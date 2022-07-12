// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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

pub mod fmt;
pub mod fs;
pub mod io;
pub mod util;

use mime::Mime;
use std::{io::Error as IoError, result::Result as StdResult};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown content type")]
    UnknownContentType,

    #[error("unsupported content type")]
    UnsupportedContentType(Mime),

    #[error("unsupported import options")]
    UnsupportedImportOptions,

    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = StdResult<T, Error>;

pub mod prelude {
    pub use super::{Error, Result};
}
