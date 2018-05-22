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
/// Confidence
///////////////////////////////////////////////////////////////////////

pub type ConfidenceValue = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Confidence(pub ConfidenceValue);

impl From<ConfidenceValue> for Confidence {
    fn from(from: ConfidenceValue) -> Self {
        Confidence(from)
    }
}

impl From<Confidence> for ConfidenceValue {
    fn from(from: Confidence) -> Self {
        from.0
    }
}

impl Deref for Confidence {
    type Target = ConfidenceValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Confidence {
    pub const MIN: Self = Confidence(0 as ConfidenceValue);

    pub const MAX: Self = Confidence(1 as ConfidenceValue);

    pub fn is_valid(&self) -> bool {
        (*self >= Self::MIN) && (*self <= Self::MAX)
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:.1}%",
            (self.0 * (1000 as ConfidenceValue)).round() / (10 as ConfidenceValue)
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

    pub confidence: Confidence,
}

impl Tag {
    pub fn new<T: Into<String>, C: Into<Confidence>>(term: T, confidence: C) -> Self {
        Self {
            facet: None,
            term: term.into(),
            confidence: confidence.into(),
        }
    }

    pub fn new_faceted<T: Into<String>, C: Into<Confidence>>(
        facet: &str,
        term: T,
        confidence: C,
    ) -> Self {
        Self {
            facet: Some(facet.to_lowercase()),
            term: term.into(),
            confidence: confidence.into(),
        }
    }

    pub fn is_faceted(&self) -> bool {
        self.facet.is_some()
    }

    pub fn is_valid(&self) -> bool {
        if !self.confidence.is_valid() {
            false
        } else if self.term.is_empty() {
            false
        } else if let Some(ref facet) = self.facet {
            !facet.is_empty() && (facet.find(char::is_uppercase) == None)
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

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagCount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facet: Option<String>,

    pub term: String,

    pub count: usize,
}

///////////////////////////////////////////////////////////////////////
/// Rating
///////////////////////////////////////////////////////////////////////

pub type RatingScore = Confidence;
pub type RatingScoreValue = ConfidenceValue;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Rating {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    pub score: RatingScore,
}

impl Rating {
    pub const MIN_SCORE: RatingScore = RatingScore::MIN;
    pub const MAX_SCORE: RatingScore = RatingScore::MAX;

    pub fn new<O: Into<String>, S: Into<RatingScore>>(owner: O, score: S) -> Self {
        Self {
            owner: Some(owner.into()),
            score: score.into(),
        }
    }

    pub fn new_anonymous<S: Into<RatingScore>>(score: S) -> Self {
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

    pub fn rating_from_stars(stars: u8, max_stars: u8) -> RatingScore {
        Confidence((stars.min(max_stars) as RatingScoreValue) / (max_stars as RatingScoreValue))
    }

    pub fn star_rating(&self, max_stars: u8) -> u8 {
        ((*self.score * (max_stars as RatingScoreValue)).ceil() as u8).min(max_stars)
    }

    pub fn minmax<'a>(
        ratings: &[Self],
        owner: Option<&'a str>,
    ) -> Option<(RatingScore, RatingScore)> {
        let count = ratings
            .iter()
            .filter(|rating| {
                owner.is_none() || rating.owner.is_none()
                    || rating.owner.as_ref().map(|owner| owner.as_str()) == owner
            })
            .count();
        if count > 0 {
            let (mut min_score, mut max_score) = (*Self::MAX_SCORE, *Self::MIN_SCORE);
            ratings
                .iter()
                .filter(|rating| {
                    owner.is_none() || rating.owner.is_none()
                        || rating.owner.as_ref().map(|owner| owner.as_str()) == owner
                })
                .for_each(|rating| {
                    min_score = min_score.min(*rating.score);
                    max_score = max_score.max(*rating.score);
                });
            Some((min_score.into(), max_score.into()))
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
    fn confidence_valid() {
        assert!(Confidence::MIN.is_valid());
        assert!(Confidence::MAX.is_valid());
        assert!(!Confidence(*Confidence::MIN - *Confidence::MAX).is_valid());
        assert!(!Confidence(*Confidence::MAX + *Confidence::MAX).is_valid());
    }

    #[test]
    fn confidence_display() {
        assert_eq!("0.0%", format!("{}", Confidence::MIN));
        assert_eq!("100.0%", format!("{}", Confidence::MAX));
        assert_eq!("90.1%", format!("{}", Confidence(0.9012345)));
        assert_eq!("90.2%", format!("{}", Confidence(0.9015)));
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
