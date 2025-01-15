// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn sample_rate_default() {
    assert_eq!(SampleRateHz::ZERO, SampleRateHz::default());
}

#[test]
fn sample_rate_is_valid() {
    assert!(!SampleRateHz::ZERO.is_valid());
    assert!(SampleRateHz::MIN.is_valid());
    assert!(SampleRateHz::MAX.is_valid());
    assert!(!SampleRateHz::default().is_valid());
    assert!(SampleRateHz::new(44_100.0).is_valid());
    assert!(SampleRateHz::new(48_000.0).is_valid());
    assert!(SampleRateHz::new(96_000.0).is_valid());
    assert!(SampleRateHz::new(192_000.0).is_valid());
}

#[test]
fn sample_rate_to_string() {
    assert!(SampleRateHz::default()
        .to_string()
        .ends_with(SampleRateHz::UNIT_OF_MEASURE));
    assert!(SampleRateHz::new(44_100.0)
        .to_string()
        .ends_with(SampleRateHz::UNIT_OF_MEASURE));
    assert!(SampleRateHz::new(48_000.0)
        .to_string()
        .ends_with(SampleRateHz::UNIT_OF_MEASURE));
    assert!(SampleRateHz::new(96_000.0)
        .to_string()
        .ends_with(SampleRateHz::UNIT_OF_MEASURE));
    assert!(SampleRateHz::new(192_000.0)
        .to_string()
        .ends_with(SampleRateHz::UNIT_OF_MEASURE));
}

#[test]
fn loudness_to_string() {
    assert!(LoudnessLufs(-1.234)
        .to_string()
        .ends_with(LoudnessLufs::UNIT_OF_MEASURE));
}
