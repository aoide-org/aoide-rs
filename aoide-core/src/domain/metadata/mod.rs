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
    pub const MIN: Confidence = Confidence(0 as ConfidenceValue);

    pub const MAX: Confidence = Confidence(1 as ConfidenceValue);

    pub fn is_valid(&self) -> bool {
        (*self >= Self::MIN) && (*self <= Self::MAX)
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.1}%", (self.0 * (1000 as ConfidenceValue)).round() / (10 as ConfidenceValue))
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

///////////////////////////////////////////////////////////////////////
/// Rating
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Rating {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    
    pub rating: Confidence,
}

impl Rating {
    pub fn new<O: Into<String>, R: Into<Confidence>>(owner: O, rating: R) -> Self {
        Self {
            owner: Some(owner.into()),
            rating: rating.into(),
        }
    }

    pub fn new_anonymous<R: Into<Confidence>>(rating: R) -> Self {
        Self {
            owner: None,
            rating: rating.into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        if !self.rating.is_valid() {
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

    pub fn rating_from_stars(stars: u8, max_stars: u8) -> Confidence {
        Confidence((stars.min(max_stars) as ConfidenceValue) / (max_stars as ConfidenceValue))
    }

    pub fn star_rating(&self, max_stars: u8) -> u8 {
        ((*self.rating * (max_stars as ConfidenceValue)).ceil() as u8).min(max_stars)
    }

    pub fn minmax<'a>(ratings: &[Self], owner: Option<&'a str>) -> Option<(Confidence, Confidence)> {
        let count = ratings
            .iter()
            .filter(|rating| owner.is_none() || rating.owner.is_none() || rating.owner.as_ref().map(|owner| owner.as_str()) == owner)
            .count();
        if count > 0 {
            let (mut min_rating, mut max_rating) = (*Confidence::MAX, *Confidence::MIN);
            ratings
                .iter()
                .filter(|rating| owner.is_none() || rating.owner.is_none() || rating.owner.as_ref().map(|owner| owner.as_str()) == owner)
                .for_each(|rating| {
                    min_rating = min_rating.min(*rating.rating);
                    max_rating = max_rating.max(*rating.rating);
                    });
            Some((Confidence(min_rating), Confidence(max_rating)))
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
    
    pub comment: String,
}

impl Comment {
    pub fn new<O: Into<String>, C: Into<String>>(owner: O, comment: C) -> Self {
        Self {
            owner: Some(owner.into()),
            comment: comment.into(),
        }
    }

    pub fn new_anonymous<C: Into<String>>(comment: C) -> Self {
        Self {
            owner: None,
            comment: comment.into(),
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
            Rating { owner: Some(owner1.into()), rating: 0.5.into() },
            Rating { owner: None, rating: Confidence(0.4) },
            Rating { owner: Some(owner2.into()), rating: Confidence(0.8) },
            Rating { owner: Some(owner3.into()), rating: Confidence(0.1) },
        ];
        assert_eq!(None, Rating::minmax(&vec![], None));
        assert_eq!(None, Rating::minmax(&vec![], Some(owner1)));
        assert_eq!(None, Rating::minmax(&vec![], Some(owner4)));
        assert_eq!(Some((Confidence(0.1), Confidence(0.8))), Rating::minmax(&ratings, None)); // all ratings
        assert_eq!(Some((Confidence(0.4), Confidence(0.5))), Rating::minmax(&ratings, Some(owner1))); // anonymous and own rating
        assert_eq!(Some((Confidence(0.4), Confidence(0.8))), Rating::minmax(&ratings, Some(owner2))); // anonymous and own rating
        assert_eq!(Some((Confidence(0.1), Confidence(0.4))), Rating::minmax(&ratings, Some(owner3))); // anonymous and own rating
        assert_eq!(Some((Confidence(0.4), Confidence(0.4))), Rating::minmax(&ratings, Some(owner4))); // only anonymous rating
    }
}
