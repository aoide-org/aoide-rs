// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod collection;
pub mod media;
pub mod playlist;
pub mod track;

#[cfg(test)]
mod tests {
    use aoide_core::{
        collection::MediaSourceConfig,
        media::content::{ContentPathConfig, VirtualFilePathConfig},
        util::url::BaseUrl,
    };

    pub(crate) fn vfs_media_source_config() -> MediaSourceConfig {
        MediaSourceConfig {
            content_path: ContentPathConfig::VirtualFilePath(VirtualFilePathConfig {
                root_url: BaseUrl::parse_strict("file:///").unwrap(),
                excluded_paths: vec![],
            }),
        }
    }
}
