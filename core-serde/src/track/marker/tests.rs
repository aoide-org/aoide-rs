// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

///////////////////////////////////////////////////////////////////////

use super::*;

use aoide_core::audio::{sample::SamplePosition, PositionMs};

#[test]
fn deserialize_millis() {
    let millis = 1001.75;
    let json = format!("{}", millis);
    let position: Position = serde_json::from_str(&json).unwrap();
    assert_eq!(Position::Millis(PositionMs(millis).into()), position);
    assert_eq!(json, serde_json::to_string(&position).unwrap());
}

#[test]
fn should_fail_to_deserialize_single_element_array_with_millis() {
    let millis = 1001.75;
    let json = format!("[{}]", millis);
    assert!(serde_json::from_str::<Position>(&json).is_err());
}

#[test]
fn deserialize_millis_samples() {
    let millis = 1001.75;
    let samples = 44177175.5;
    let json = format!("[{},{}]", millis, samples);
    let position: Position = serde_json::from_str(&json).unwrap();
    assert_eq!(
        Position::MillisSamples(PositionMs(millis).into(), SamplePosition(samples).into()),
        position
    );
    assert_eq!(json, serde_json::to_string(&position).unwrap());
}

#[test]
fn deserialize_hotcue_end_marker() {
    let millis = 1001.75;
    let samples = 44177175.5;
    let end_millis = 1234.0;
    let json = format!(
        "{{\"pos\":[{millis},{samples}],\"end\":1234.0,\"typ\":1}}",
        millis = millis,
        samples = samples,
    );
    let beatloop_marker: CueMarker = serde_json::from_str(&json).unwrap();
    assert_eq!(
        CueMarker {
            r#type: CueMarkerType::HotCue,
            start: Some(Position::MillisSamples(
                PositionMs(millis).into(),
                SamplePosition(samples).into()
            )),
            extent: Some(MarkerExtent::EndPosition(Position::Millis(
                PositionMs(f64::from(end_millis)).into()
            ))),
            out_behavior: None,
            color: None,
            number: None,
            label: None,
        },
        beatloop_marker
    );
    assert_eq!(json, serde_json::to_string(&beatloop_marker).unwrap());
}

#[test]
fn deserialize_hotcue_beatloop_marker() {
    let millis = 1001.75;
    let samples = 44177175.5;
    let beatloop_len_x32 = 8; // 1/4 beat
    let json = format!(
        "{{\"pos\":[{millis},{samples}],\"b32\":{b32},\"out\":2,\"typ\":1}}",
        millis = millis,
        samples = samples,
        b32 = beatloop_len_x32,
    );
    let beatloop_marker: CueMarker = serde_json::from_str(&json).unwrap();
    assert_eq!(
        CueMarker {
            r#type: CueMarkerType::HotCue,
            start: Some(Position::MillisSamples(
                PositionMs(millis).into(),
                SamplePosition(samples).into()
            )),
            extent: Some(MarkerExtent::BeatCountX32(beatloop_len_x32)),
            out_behavior: Some(OutBehavior::Loop),
            color: None,
            number: None,
            label: None,
        },
        beatloop_marker
    );
    assert_eq!(json, serde_json::to_string(&beatloop_marker).unwrap());
}
