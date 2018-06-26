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
fn channel_count_default() {
    assert!(!ChannelCount::default().is_valid());
}

#[test]
fn channel_count_minmax() {
    assert!(ChannelCount::MIN.is_valid());
    assert!(ChannelCount::MAX.is_valid());
}

#[test]
fn channel_layout_channel_count() {
    assert_eq!(ChannelLayout::Mono.channel_count(), ChannelCount::new(1));
    assert_eq!(
        ChannelLayout::DualMono.channel_count(),
        ChannelCount::new(2)
    );
    assert_eq!(ChannelLayout::Stereo.channel_count(), ChannelCount::new(2));
}

#[test]
fn channels_default() {
    assert_eq!(ChannelCount::default(), Channels::default().count);
}

#[test]
fn channels_is_valid() {
    assert!(!Channels::default().is_valid());
    assert!(Channels::layout(ChannelLayout::Mono).is_valid());
    assert!(Channels::layout(ChannelLayout::DualMono).is_valid());
    assert!(Channels::layout(ChannelLayout::Stereo).is_valid());
    assert!(Channels::count(ChannelCount::MIN).is_valid());
    assert!(Channels::count(ChannelCount::MAX).is_valid());
    assert!(!Channels {
        count: ChannelCount::new(1),
        layout: Some(ChannelLayout::DualMono),
    }.is_valid());
    assert!(!Channels {
        count: ChannelCount::new(2),
        layout: Some(ChannelLayout::Mono),
    }.is_valid());
    assert!(!Channels {
        count: ChannelCount::new(3),
        layout: Some(ChannelLayout::Stereo),
    }.is_valid());
}

#[test]
fn channel_count_default_layout() {
    assert_eq!(None, Channels::default_layout(ChannelCount::default()));
    assert_eq!(
        Some(ChannelLayout::Mono),
        Channels::default_layout(ChannelCount::new(1))
    );
    assert_eq!(
        Some(ChannelLayout::Stereo),
        Channels::default_layout(ChannelCount::new(2))
    );
    assert_eq!(None, Channels::default_layout(ChannelCount::new(3)));
}

#[test]
fn duration_default() {
    assert_eq!(DurationMs::EMPTY, DurationMs::default());
}

#[test]
fn duration_to_string() {
    assert!(
        DurationMs::default()
            .to_string()
            .ends_with(DurationMs::UNIT_OF_MEASURE)
    );
}

#[test]
fn loudness_to_string() {
    assert!(
        Loudness::EbuR128(LufsDb::default())
            .to_string()
            .ends_with(LufsDb::UNIT_OF_MEASURE)
    );
}
