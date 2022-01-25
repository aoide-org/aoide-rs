// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
