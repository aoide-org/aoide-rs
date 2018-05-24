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

use std::fmt;

use std::ops::Deref;

///////////////////////////////////////////////////////////////////////
/// Scoring
///////////////////////////////////////////////////////////////////////

pub type ScoreValue = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Score(pub ScoreValue);

impl From<ScoreValue> for Score {
    fn from(from: ScoreValue) -> Self {
        Score(from)
    }
}

impl From<Score> for ScoreValue {
    fn from(from: Score) -> Self {
        from.0
    }
}

impl Deref for Score {
    type Target = ScoreValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Score {
    pub const MIN: Self = Score(0 as ScoreValue);

    pub const MAX: Self = Score(1 as ScoreValue);

    pub fn new<S: Into<ScoreValue>>(score_value: S) -> Self {
        score_value.into().min(*Self::MAX).max(*Self::MIN).into()
    }

    pub fn is_valid(&self) -> bool {
        (*self >= Self::MIN) && (*self <= Self::MAX)
    }

    pub fn is_min(&self) -> bool {
        *self <= Self::MIN
    }

    pub fn is_max(&self) -> bool {
        *self >= Self::MAX
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        debug_assert!(self.is_valid());
        write!(
            f,
            "{:.1}%",
            (self.0 * (1000 as ScoreValue)).round() / (10 as ScoreValue)
        )
    }
}

///////////////////////////////////////////////////////////////////////
/// Tag
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Tag {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facet: Option<String>, // lowercase / case-insensitive

    pub term: String,

    #[serde(skip_serializing_if = "Tag::is_default_score", default = "Tag::default_score")]
    pub score: Score,
}

impl Tag {
    pub fn default_score() -> Score {
        Score::MAX
    }

    pub fn is_default_score(score: &Score) -> bool {
        *score == Self::default_score()
    }

    pub fn new<T: Into<String>, S: Into<Score>>(term: T, score: S) -> Self {
        Self {
            facet: None,
            term: term.into(),
            score: score.into(),
        }
    }

    pub fn new_faceted<F: AsRef<str>, T: Into<String>, S: Into<Score>>(
        facet: F,
        term: T,
        score: S,
    ) -> Self {
        Self {
            facet: Some(facet.as_ref().to_lowercase()),
            term: term.into(),
            score: score.into(),
        }
    }

    pub fn is_faceted(&self) -> bool {
        self.facet.is_some()
    }

    pub fn is_valid(&self) -> bool {
        if !self.score.is_valid() || self.term.is_empty() {
            false
        } else if let Some(ref facet) = self.facet {
            !facet.is_empty() && !facet.contains(char::is_uppercase)
        } else {
            true
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagFacetCount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facet: Option<String>,

    pub count: usize,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MultiTag {
    pub tag: Tag,

    pub count: usize,
}

///////////////////////////////////////////////////////////////////////
/// Rating
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Rating {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    pub score: Score,
}

impl Rating {
    pub fn new<O: Into<String>, S: Into<Score>>(owner: O, score: S) -> Self {
        Self {
            owner: Some(owner.into()),
            score: score.into(),
        }
    }

    pub fn new_anonymous<S: Into<Score>>(score: S) -> Self {
        Self {
            owner: None,
            score: score.into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        if !self.score.is_valid() {
            false
        } else if let Some(ref owner) = self.owner {
            !owner.is_empty()
        } else {
            true
        }
    }

    pub fn is_anonymous(&self) -> bool {
        self.owner.is_none()
    }

    pub fn rating_from_stars(stars: u8, max_stars: u8) -> Score {
        Score((stars.min(max_stars) as ScoreValue) / (max_stars as ScoreValue))
    }

    pub fn star_rating(&self, max_stars: u8) -> u8 {
        ((*self.score * (max_stars as ScoreValue)).ceil() as u8).min(max_stars)
    }

    pub fn minmax<'a>(
        ratings: &[Self],
        owner: Option<&'a str>,
    ) -> Option<(Score, Score)> {
        let count = ratings
            .iter()
            .filter(|rating| {
                owner.is_none() || rating.owner.is_none()
                    || rating.owner.as_ref().map(|owner| owner.as_str()) == owner
            })
            .count();
        if count > 0 {
            let (mut score_min, mut score_max) = (*Score::MAX, *Score::MIN);
            ratings
                .iter()
                .filter(|rating| {
                    owner.is_none() || rating.owner.is_none()
                        || rating.owner.as_ref().map(|owner| owner.as_str()) == owner
                })
                .for_each(|rating| {
                    score_min = score_min.min(*rating.score);
                    score_max = score_max.max(*rating.score);
                });
            Some((score_min.into(), score_max.into()))
        } else {
            None
        }
    }
}

///////////////////////////////////////////////////////////////////////
/// Comment
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Comment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    pub text: String,
}

impl Comment {
    pub fn new<O: Into<String>, T: Into<String>>(owner: O, text: T) -> Self {
        Self {
            owner: Some(owner.into()),
            text: text.into(),
        }
    }

    pub fn new_anonymous<T: Into<String>>(text: T) -> Self {
        Self {
            owner: None,
            text: text.into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        if let Some(ref owner) = self.owner {
            !owner.is_empty()
        } else {
            true
        }
    }

    pub fn is_anonymous(&self) -> bool {
        self.owner.is_none()
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_valid() {
        assert!(Score::MIN.is_valid());
        assert!(Score::MAX.is_valid());
        assert!(Score::MIN.is_min());
        assert!(!Score::MAX.is_min());
        assert!(!Score::MIN.is_max());
        assert!(Score::MAX.is_max());
        assert!(Score(*Score::MIN + *Score::MAX).is_valid());
        assert!(!Score(*Score::MIN - *Score::MAX).is_valid());
        assert!(Score(*Score::MIN - *Score::MAX).is_min());
        assert!(!Score(*Score::MAX + *Score::MAX).is_valid());
        assert!(Score(*Score::MAX + *Score::MAX).is_max());
    }

    #[test]
    fn score_display() {
        assert_eq!("0.0%", format!("{}", Score::MIN));
        assert_eq!("100.0%", format!("{}", Score::MAX));
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
            Rating {
                owner: Some(owner1.into()),
                score: 0.5.into(),
            },
            Rating {
                owner: None,
                score: 0.4.into(),
            },
            Rating {
                owner: Some(owner2.into()),
                score: 0.8.into(),
            },
            Rating {
                owner: Some(owner3.into()),
                score: 0.1.into(),
            },
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
}
