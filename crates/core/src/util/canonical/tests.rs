// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

impl CanonicalOrd for String {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

impl IsCanonical for String {
    fn is_canonical(&self) -> bool {
        self.chars().all(char::is_lowercase)
    }
}

impl Canonicalize for String {
    fn canonicalize(&mut self) {
        let mut canonicalized = self.to_lowercase();
        std::mem::swap(self, &mut canonicalized);
    }
}

#[test]
fn canonicalize_vec() {
    assert_eq!(
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
        vec![
            "B".to_string(),
            "A".to_string(),
            "c".to_string(),
            "a".to_string(),
            "C".to_string(),
            "b".to_string(),
            "c".to_string(),
            "b".to_string(),
            "a".to_string(),
            "A".to_string(),
            "B".to_string(),
        ]
        .canonicalize_into()
    );
}
