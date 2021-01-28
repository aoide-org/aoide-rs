// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

///////////////////////////////////////////////////////////////////////

use bytes::BufMut as _;
use digest::Digest;
use sha2::Sha256;
use std::{
    ffi::OsStr,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

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

#[derive(Debug, Default)]
pub struct MediaDigest {
    default_blake3: Option<blake3::Hasher>,
    legacy_sha256: Option<Sha256>,
}

impl MediaDigest {
    pub const fn digest_size() -> usize {
        32
    }

    pub fn new() -> Self {
        Self {
            default_blake3: Some(blake3::Hasher::new()),
            legacy_sha256: None,
        }
    }

    pub fn sha256() -> Self {
        Self {
            default_blake3: Some(blake3::Hasher::new()),
            legacy_sha256: Some(Sha256::new()),
        }
    }

    pub fn digest_content(&mut self, content_data: &[u8]) -> Option<[u8; Self::digest_size()]> {
        let Self {
            default_blake3,
            legacy_sha256,
        } = self;
        if let Some(digest) = default_blake3 {
            // Default
            digest.update(content_data);
            Some(digest.finalize_reset().into())
        } else {
            // Legacy
            legacy_sha256.as_mut().map(|digest| {
                digest.update(content_data);
                digest.finalize_reset().into()
            })
        }
    }
}
