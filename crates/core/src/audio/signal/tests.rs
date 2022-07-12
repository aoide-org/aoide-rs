// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

///////////////////////////////////////////////////////////////////////

#[test]
fn sample_rate_default() {
    assert!(SampleRateHz::default().validate().is_err());
}

#[test]
fn validate_sample_rate() {
    assert!(SampleRateHz::min().validate().is_ok());
    assert!(SampleRateHz::max().validate().is_ok());
    assert!(SampleRateHz::of_compact_disc().validate().is_ok());
    assert!(SampleRateHz::of_studio_48k().validate().is_ok());
    assert!(SampleRateHz::of_studio_96k().validate().is_ok());
    assert!(SampleRateHz::of_studio_192k().validate().is_ok());
}

#[test]
fn sample_rate_to_string() {
    assert!(SampleRateHz::default()
        .to_string()
        .ends_with(SampleRateHz::unit_of_measure()));
    assert!(SampleRateHz::of_compact_disc()
        .to_string()
        .ends_with(SampleRateHz::unit_of_measure()));
    assert!(SampleRateHz::of_studio_48k()
        .to_string()
        .ends_with(SampleRateHz::unit_of_measure()));
    assert!(SampleRateHz::of_studio_96k()
        .to_string()
        .ends_with(SampleRateHz::unit_of_measure()));
    assert!(SampleRateHz::of_studio_192k()
        .to_string()
        .ends_with(SampleRateHz::unit_of_measure()));
}

#[test]
fn loudness_to_string() {
    assert!(LoudnessLufs(-1.234)
        .to_string()
        .ends_with(LoudnessLufs::unit_of_measure()));
}
