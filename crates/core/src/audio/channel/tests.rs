// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn channel_count_default() {
    assert!(ChannelCount::default().validate().is_err());
}

#[test]
fn channel_count_validate() {
    assert!(ChannelCount::MIN.validate().is_ok());
    assert!(ChannelCount::MAX.validate().is_ok());
    // Support more than 255 channels
    assert!(ChannelCount(256).validate().is_ok());
}

#[test]
fn channel_flags_default() {
    assert!(ChannelFlags::default().validate().is_err());
}

#[test]
fn channel_flags_validate() {
    assert!(ChannelFlags::empty().validate().is_err());
    assert!(ChannelFlags::all().validate().is_ok());
    assert!(
        ChannelFlags::from_bits_retain(ChannelFlags::all().bits() >> 1)
            .validate()
            .is_ok()
    );
    assert!(
        ChannelFlags::from_bits_retain(ChannelFlags::all().bits() << 1)
            .validate()
            .is_err()
    );
}
