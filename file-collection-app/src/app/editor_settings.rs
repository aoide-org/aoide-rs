// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Persistent settings of the track editor and the track selection editor.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "editor_settings.json";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct EditorSettings {
    /// Also write the files when saving the edited tracks.
    pub(crate) write_files: bool,
}

impl EditorSettings {
    #[must_use]
    fn from_json(json: &[u8]) -> Option<Self> {
        serde_json::from_slice(json).ok()
    }

    #[must_use]
    fn to_json(self) -> Vec<u8> {
        serde_json::to_vec_pretty(&self).expect("infallible")
    }

    #[must_use]
    fn file_path(settings_dir: &Path) -> PathBuf {
        settings_dir.join(FILE_NAME)
    }

    /// Load the settings.
    ///
    /// Falls back to the defaults if the settings have not been saved
    /// yet or could not be restored.
    #[must_use]
    pub(crate) fn restore(settings_dir: &Path) -> Self {
        let file_path = Self::file_path(settings_dir);
        let json = match fs::read(&file_path) {
            Ok(json) => json,
            Err(err) => {
                if err.kind() != io::ErrorKind::NotFound {
                    log::warn!(
                        "Failed to read editor settings from file '{file_path}': {err}",
                        file_path = file_path.display(),
                    );
                }
                return Self::default();
            }
        };
        Self::from_json(&json).unwrap_or_else(|| {
            log::warn!(
                "Ignoring invalid editor settings in file '{file_path}'",
                file_path = file_path.display(),
            );
            Self::default()
        })
    }

    pub(crate) fn save(self, settings_dir: &Path) -> io::Result<()> {
        fs::write(Self::file_path(settings_dir), self.to_json())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip() {
        for write_files in [false, true] {
            let settings = EditorSettings { write_files };
            assert_eq!(
                Some(settings),
                EditorSettings::from_json(&settings.to_json())
            );
        }
    }

    #[test]
    fn from_json_rejects_invalid_contents() {
        assert_eq!(None, EditorSettings::from_json(b"not json"));
        assert_eq!(None, EditorSettings::from_json(b"{}"));
        assert_eq!(
            None,
            EditorSettings::from_json(br#"{"write_files": "yes"}"#)
        );
    }

    #[test]
    fn saved_settings_are_restored() {
        let settings_dir = std::env::temp_dir().join(format!(
            "aoide_editor_settings_test_{pid}",
            pid = std::process::id()
        ));
        fs::create_dir_all(&settings_dir).unwrap();
        assert_eq!(
            EditorSettings::default(),
            EditorSettings::restore(&settings_dir)
        );
        let settings = EditorSettings { write_files: true };
        settings.save(&settings_dir).unwrap();
        assert_eq!(settings, EditorSettings::restore(&settings_dir));
        // Invalid contents are replaced by the defaults.
        fs::write(EditorSettings::file_path(&settings_dir), b"invalid").unwrap();
        assert_eq!(
            EditorSettings::default(),
            EditorSettings::restore(&settings_dir)
        );
        fs::remove_dir_all(&settings_dir).unwrap();
    }
}
