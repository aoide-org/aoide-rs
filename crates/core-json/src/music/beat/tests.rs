// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn deserialize_top() {
    let top = 4;
    let json = top.to_string();
    let timing: TimeSignature = serde_json::from_str(&json).unwrap();
    assert_eq!(TimeSignature::Top(top), timing);
    assert_eq!(json, serde_json::to_string(&timing).unwrap());
}

#[test]
fn should_fail_to_deserialize_single_element_array_with_top() {
    let top = 4;
    let json = format!("[{top}]");
    assert!(serde_json::from_str::<TimeSignature>(&json).is_err());
}

#[test]
fn deserialize_top_bottom() {
    let top = 3;
    let bottom = 4;
    let json = format!("[{top},{bottom}]");
    let timing: TimeSignature = serde_json::from_str(&json).unwrap();
    assert_eq!(TimeSignature::TopBottom(top, bottom), timing);
    assert_eq!(json, serde_json::to_string(&timing).unwrap());
}
