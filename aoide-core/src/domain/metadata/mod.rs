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

#[cfg(test)]
mod tests;

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
pub struct ScoredTag(
    /*score*/ Score,
    /*term*/ String,
    /*facet*/ Option<String>, // lowercase / case-insensitive
);

impl ScoredTag {
    pub fn default_score() -> Score {
        Score::MAX
    }

    pub fn is_default_score(score: &Score) -> bool {
        *score == Self::default_score()
    }

    pub fn new<S: Into<Score>, T: Into<String>, F: Into<String>>(
        score: S,
        term: T,
        facet: Option<F>,
    ) -> Self {
        ScoredTag(score.into(), term.into(), facet.map(F::into))
    }

    pub fn new_term<S: Into<Score>, T: Into<String>>(score: S, term: T) -> Self {
        ScoredTag(score.into(), term.into(), None)
    }

    pub fn new_faceted_term<S: Into<Score>, T: Into<String>, F: Into<String>>(
        score: S,
        term: T,
        facet: F,
    ) -> Self {
        ScoredTag(score.into(), term.into(), Some(facet.into()))
    }

    pub fn score(&self) -> Score {
        self.0
    }

    pub fn term<'a>(&'a self) -> &'a String {
        &self.1
    }

    pub fn facet<'a>(&'a self) -> &'a Option<String> {
        &self.2
    }

    pub fn is_faceted(&self) -> bool {
        self.facet().is_some()
    }

    pub fn is_valid(&self) -> bool {
        if !self.score().is_valid() || self.term().is_empty() {
            false
        } else if let Some(ref facet) = self.facet().as_ref() {
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
pub struct ScoredTagCount {
    pub tag: ScoredTag,

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
