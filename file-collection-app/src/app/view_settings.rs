// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Persistent settings of the appearance.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use egui::{Color32, Context, Theme};
use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "view_settings.json";

/// Font size relative to the default font size in percent.
pub(crate) const DEFAULT_FONT_SIZE_PERCENT: u16 = 100;
pub(crate) const MIN_FONT_SIZE_PERCENT: u16 = 50;
pub(crate) const MAX_FONT_SIZE_PERCENT: u16 = 300;
pub(crate) const FONT_SIZE_PERCENT_STEP: u16 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ViewSettings {
    /// Font size relative to the default font size in percent.
    pub(crate) font_size_percent: u16,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            font_size_percent: DEFAULT_FONT_SIZE_PERCENT,
        }
    }
}

#[must_use]
pub(crate) fn clamp_font_size_percent(font_size_percent: u16) -> u16 {
    font_size_percent.clamp(MIN_FONT_SIZE_PERCENT, MAX_FONT_SIZE_PERCENT)
}

impl ViewSettings {
    #[must_use]
    fn from_json(json: &[u8]) -> Option<Self> {
        let Self { font_size_percent } = serde_json::from_slice(json).ok()?;
        Some(Self {
            font_size_percent: clamp_font_size_percent(font_size_percent),
        })
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
                        "Failed to read view settings from file '{file_path}': {err}",
                        file_path = file_path.display(),
                    );
                }
                return Self::default();
            }
        };
        Self::from_json(&json).unwrap_or_else(|| {
            log::warn!(
                "Ignoring invalid view settings in file '{file_path}'",
                file_path = file_path.display(),
            );
            Self::default()
        })
    }

    pub(crate) fn save(self, settings_dir: &Path) -> io::Result<()> {
        fs::write(Self::file_path(settings_dir), self.to_json())
    }

    /// Apply the settings to the styles of all themes.
    pub(crate) fn apply(self, ctx: &Context) {
        let Self { font_size_percent } = self;
        let scale = f32::from(font_size_percent) / 100.0;
        let text_styles = egui::style::default_text_styles()
            .into_iter()
            .map(|(text_style, mut font_id)| {
                font_id.size *= scale;
                (text_style, font_id)
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        ctx.all_styles_mut(|style| {
            style.text_styles.clone_from(&text_styles);
        });
    }
}

/// Increase the contrast of the text in dark mode.
///
/// The default dark theme of egui renders normal text with a dim gray
/// that is hardly readable.
pub(crate) fn increase_dark_mode_contrast(ctx: &Context) {
    ctx.style_mut_of(Theme::Dark, |style| {
        let visuals = &mut style.visuals;
        // Normal, non-interactive text (labels).
        visuals.widgets.noninteractive.fg_stroke.color = Color32::from_gray(220);
        // Text of buttons and other interactive widgets.
        visuals.widgets.inactive.fg_stroke.color = Color32::from_gray(235);
        visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
        visuals.widgets.active.fg_stroke.color = Color32::WHITE;
        visuals.widgets.open.fg_stroke.color = Color32::WHITE;
        // Hints and other secondary text.
        visuals.weak_text_color = Some(Color32::from_gray(170));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip() {
        for font_size_percent in [
            MIN_FONT_SIZE_PERCENT,
            DEFAULT_FONT_SIZE_PERCENT,
            MAX_FONT_SIZE_PERCENT,
        ] {
            let settings = ViewSettings { font_size_percent };
            assert_eq!(Some(settings), ViewSettings::from_json(&settings.to_json()));
        }
    }

    #[test]
    fn from_json_clamps_font_size() {
        assert_eq!(
            Some(ViewSettings {
                font_size_percent: MAX_FONT_SIZE_PERCENT
            }),
            ViewSettings::from_json(br#"{"font_size_percent": 1000}"#)
        );
        assert_eq!(
            Some(ViewSettings {
                font_size_percent: MIN_FONT_SIZE_PERCENT
            }),
            ViewSettings::from_json(br#"{"font_size_percent": 0}"#)
        );
    }

    #[test]
    fn from_json_rejects_invalid_contents() {
        assert_eq!(None, ViewSettings::from_json(b"not json"));
        assert_eq!(None, ViewSettings::from_json(b"{}"));
        assert_eq!(
            None,
            ViewSettings::from_json(br#"{"font_size_percent": "large"}"#)
        );
    }
}
