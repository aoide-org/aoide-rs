// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

use super::*;

#[test]
fn channel_layout_channel_count() {
    assert_eq!(ChannelLayout::Mono.channel_count(), 1);
    assert_eq!(ChannelLayout::DualMono.channel_count(), 2);
    assert_eq!(ChannelLayout::Stereo.channel_count(), 2);
}

#[test]
fn channels_default() {
    assert_eq!(0, Channels::default().count);
}

#[test]
fn channels_is_valid() {
    assert!(!Channels::default().is_valid());
    assert!(Channels::layout(ChannelLayout::Mono).is_valid());
    assert!(Channels::layout(ChannelLayout::DualMono).is_valid());
    assert!(Channels::layout(ChannelLayout::Stereo).is_valid());
    assert!(Channels::count(Channels::COUNT_MIN).is_valid());
    assert!(Channels::count(Channels::COUNT_MAX).is_valid());
    assert!(!Channels {
        count: 1,
        layout: Some(ChannelLayout::DualMono),
    }.is_valid());
    assert!(!Channels {
        count: 2,
        layout: Some(ChannelLayout::Mono),
    }.is_valid());
    assert!(!Channels {
        count: 3,
        layout: Some(ChannelLayout::Stereo),
    }.is_valid());
}

#[test]
fn channel_count_default_layout() {
    assert_eq!(None, Channels::default_layout(0));
    assert_eq!(Some(ChannelLayout::Mono), Channels::default_layout(1));
    assert_eq!(Some(ChannelLayout::Stereo), Channels::default_layout(2));
    assert_eq!(None, Channels::default_layout(3));
}
