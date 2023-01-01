use std::mem::MaybeUninit;

// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod source;
pub mod tracker;

pub const DIGEST_BYTES_LEN: usize = 32;

pub type DigestBytes = [u8; DIGEST_BYTES_LEN];

#[allow(unsafe_code)]
#[must_use]
pub fn read_digest_from_slice(bytes: &[u8]) -> Option<DigestBytes> {
    if bytes.len() == DIGEST_BYTES_LEN {
        let mut digest = MaybeUninit::<DigestBytes>::uninit();
        Some(unsafe {
            (*digest.as_mut_ptr()).copy_from_slice(&bytes[0..DIGEST_BYTES_LEN]);
            digest.assume_init()
        })
    } else {
        None
    }
}
