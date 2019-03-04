// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

///////////////////////////////////////////////////////////////////////

#[test]
fn score_valid() {
    assert!(Score::min().is_valid());
    assert!(Score::max().is_valid());
    assert!(Score::min().is_min());
    assert!(!Score::max().is_min());
    assert!(!Score::min().is_max());
    assert!(Score::max().is_max());
    assert!(Score(Score::min().0 + Score::max().0).is_valid());
    assert!(!Score(Score::min().0 - Score::max().0).is_valid());
    assert!(Score(Score::min().0 - Score::max().0).is_min());
    assert!(!Score(Score::max().0 + Score::max().0).is_valid());
    assert!(Score(Score::max().0 + Score::max().0).is_max());
}

#[test]
fn score_display() {
    assert_eq!("0.0%", format!("{}", Score::min()));
    assert_eq!("100.0%", format!("{}", Score::max()));
    assert_eq!("90.1%", format!("{}", Score(0.9012345)));
    assert_eq!("90.2%", format!("{}", Score(0.9015)));
}

#[test]
fn minmax_rating() {
    let owner1 = "a";
    let owner2 = "b";
    let owner3 = "c";
    let owner4 = "d";
    let ratings = vec![
        Rating::new_owned(0.5, owner1),
        Rating::new_anonymous(0.4),
        Rating::new_owned(0.8, owner2),
        Rating::new_owned(0.1, owner3),
    ];
    assert_eq!(None, Rating::minmax(&vec![], None));
    assert_eq!(None, Rating::minmax(&vec![], Some(owner1)));
    assert_eq!(None, Rating::minmax(&vec![], Some(owner4)));
    assert_eq!(
        Some((0.1.into(), 0.8.into())),
        Rating::minmax(&ratings, None)
    ); // all ratings
    assert_eq!(
        Some((0.4.into(), 0.5.into())),
        Rating::minmax(&ratings, Some(owner1))
    ); // anonymous and own rating
    assert_eq!(
        Some((0.4.into(), 0.8.into())),
        Rating::minmax(&ratings, Some(owner2))
    ); // anonymous and own rating
    assert_eq!(
        Some((0.1.into(), 0.4.into())),
        Rating::minmax(&ratings, Some(owner3))
    ); // anonymous and own rating
    assert_eq!(
        Some((0.4.into(), 0.4.into())),
        Rating::minmax(&ratings, Some(owner4))
    ); // only anonymous rating
}
