// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod source;
pub mod tracker;

pub const DIGEST_BYTES_LEN: usize = 32;

pub type DigestBytes = [u8; DIGEST_BYTES_LEN];

#[must_use]
pub fn read_digest_from_slice(bytes: &[u8]) -> Option<DigestBytes> {
    debug_assert_eq!(DIGEST_BYTES_LEN, bytes.len());
    bytes.try_into().ok()
}
