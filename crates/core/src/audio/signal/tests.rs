// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn sample_rate_default() {
    assert!(SampleRateHz::default().validate().is_err());
}

#[test]
fn validate_sample_rate() {
    assert!(SampleRateHz::MIN.validate().is_ok());
    assert!(SampleRateHz::MAX.validate().is_ok());
    assert!(SampleRateHz::new(44_100.0).validate().is_ok());
    assert!(SampleRateHz::new(48_000.0).validate().is_ok());
    assert!(SampleRateHz::new(96_000.0).validate().is_ok());
    assert!(SampleRateHz::new(192_000.0).validate().is_ok());
    assert!(SampleRateHz::new(0.0).validate().is_err());
    assert!(SampleRateHz::new(SampleRateHz::MAX.to_inner() + 0.1)
        .validate()
        .is_err());
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
