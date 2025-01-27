// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Stateless, cryptographically insecure random generator for simple,
//! noncritical use cases.

#[cfg(target_family = "wasm")]
pub type AdhocRng = rand::rngs::StdRng;

#[cfg(target_family = "wasm")]
#[must_use]
pub fn adhoc_rng() -> AdhocRng {
    <AdhocRng as rand::SeedableRng>::from_os_rng()
}

#[cfg(not(target_family = "wasm"))]
pub type AdhocRng = rand::rngs::ThreadRng;

#[cfg(not(target_family = "wasm"))]
#[must_use]
pub fn adhoc_rng() -> AdhocRng {
    rand::rng()
}
