// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    ffi::OsStr,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bytes::BufMut as _;
use digest::Digest;

pub fn digest_u64<D: Digest>(digest: &mut D, val: u64) {
    let mut bytes = [0u8; 8];
    let mut buf = &mut bytes[..];
    buf.put_u64(val);
    digest.update(bytes);
}

pub fn digest_u128<D: Digest>(digest: &mut D, val: u128) {
    let mut bytes = [0u8; 16];
    let mut buf = &mut bytes[..];
    buf.put_u128(val);
    digest.update(bytes);
}

pub fn digest_duration<D: Digest>(digest: &mut D, duration: Duration) {
    digest_u128(digest, duration.as_nanos());
}

#[allow(clippy::missing_panics_doc)] // Never panics
pub fn digest_system_time<D: Digest>(digest: &mut D, system_time: SystemTime) {
    digest_duration(
        digest,
        system_time
            .duration_since(UNIX_EPOCH)
            .expect("valid system time not before 1970-01-01 00:00:00 UTC"),
    );
}

pub fn digest_os_str<D: Digest>(digest: &mut D, os_str: &OsStr) {
    if let Some(utf8_str) = os_str.to_str() {
        digest.update(utf8_str.as_bytes());
    } else {
        digest.update(os_str.to_string_lossy().as_bytes());
    }
}

pub fn digest_path<D: Digest>(digest: &mut D, path: &Path) {
    digest_os_str(digest, path.as_os_str());
}

#[derive(Debug)]
pub struct MediaDigest {
    hasher: Option<blake3::Hasher>,
}

impl MediaDigest {
    #[must_use]
    pub(crate) const fn digest_size() -> usize {
        32
    }

    #[must_use]
    pub(crate) const fn dummy() -> Self {
        Self { hasher: None }
    }

    #[must_use]
    pub fn new() -> Self {
        Self {
            hasher: Some(blake3::Hasher::new()),
        }
    }

    pub fn digest_content(&mut self, content_data: &[u8]) -> &mut Self {
        self.hasher
            .as_mut()
            .map(|hasher| hasher.update(content_data));
        self
    }

    pub fn finalize_reset(&mut self) -> Option<[u8; Self::digest_size()]> {
        self.hasher
            .as_mut()
            .map(|hasher| hasher.finalize_reset().into())
    }
}

impl Default for MediaDigest {
    fn default() -> Self {
        Self::new()
    }
}
