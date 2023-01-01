// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn channel_count_default() {
    assert!(ChannelCount::default().validate().is_err());
}

#[test]
fn channel_count_minmax() {
    assert!(ChannelCount::min().validate().is_ok());
    assert!(ChannelCount::max().validate().is_ok());
}

#[test]
fn channel_layout_channel_count() {
    assert_eq!(ChannelCount(1), ChannelLayout::Mono.channel_count());
    assert_eq!(ChannelCount(2), ChannelLayout::DualMono.channel_count());
    assert_eq!(ChannelCount(2), ChannelLayout::Stereo.channel_count());
}

#[test]
fn channels_default() {
    assert_eq!(ChannelCount::default(), Channels::default().count());
}

#[test]
fn channels_validate() {
    assert!(Channels::default().validate().is_err());
    assert!(Channels::Layout(ChannelLayout::Mono).validate().is_ok());
    assert!(Channels::Layout(ChannelLayout::DualMono).validate().is_ok());
    assert!(Channels::Layout(ChannelLayout::Stereo).validate().is_ok());
    assert!(Channels::Count(ChannelCount::min()).validate().is_ok());
    assert!(Channels::Count(ChannelCount::max()).validate().is_ok());
}

#[test]
fn validate_channels() {
    assert!(Channels::default().validate().is_err());
    assert!(Channels::Layout(ChannelLayout::Mono).validate().is_ok());
    assert!(Channels::Layout(ChannelLayout::DualMono).validate().is_ok());
    assert!(Channels::Layout(ChannelLayout::Stereo).validate().is_ok());
    assert!(Channels::Count(ChannelCount::min()).validate().is_ok());
    assert!(Channels::Count(ChannelCount::max()).validate().is_ok());
}

#[test]
fn channel_count_default_layout() {
    assert_eq!(None, ChannelCount::default().default_layout());
    assert_eq!(Some(ChannelLayout::Mono), ChannelCount(1).default_layout());
    assert_eq!(
        Some(ChannelLayout::Stereo),
        ChannelCount(2).default_layout()
    );
    assert_eq!(None, ChannelCount::default_layout(ChannelCount(3)));
}
