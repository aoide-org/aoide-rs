// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

    pub fn is_min(self) -> bool {
        self <= Self::MIN
    }

    pub fn is_max(self) -> bool {
        self >= Self::MAX
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        debug_assert!(self.is_valid());
        write!(
            f,
            "{:.1}%",
            (self.0 * ScoreValue::from(1_000)).round() / ScoreValue::from(10)
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
        let score = score.into();
        let term = term.into();
        let facet = facet.map(F::into);
        debug_assert!(match facet {
            None => true,
            Some(ref facet) => facet == &facet.to_lowercase(),
        });
        ScoredTag(score, term, facet)
    }

    pub fn new_term<S: Into<Score>, T: Into<String>>(score: S, term: T) -> Self {
        let facet: Option<String> = None;
        Self::new(score, term, facet)
    }

    pub fn new_term_faceted<S: Into<Score>, T: Into<String>, F: Into<String>>(
        score: S,
        term: T,
        facet: F,
    ) -> Self {
        Self::new(score, term, Some(facet))
    }

    pub fn score(&self) -> Score {
        self.0
    }

    pub fn term(&self) -> &String {
        &self.1
    }

    pub fn facet(&self) -> &Option<String> {
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

///////////////////////////////////////////////////////////////////////
/// Rating
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Rating(/*score*/ Score, /*owner*/ Option<String>);

impl Rating {
    pub fn new<S: Into<Score>, O: Into<String>>(score: S, owner: Option<O>) -> Self {
        Rating(score.into(), owner.map(O::into))
    }

    pub fn new_anonymous<S: Into<Score>>(score: S) -> Self {
        Rating(score.into(), None)
    }

    pub fn new_owned<S: Into<Score>, O: Into<String>>(score: S, owner: O) -> Self {
        Rating(score.into(), Some(owner.into()))
    }

    pub fn score(&self) -> Score {
        self.0
    }

    pub fn owner(&self) -> &Option<String> {
        &self.1
    }

    pub fn is_valid(&self) -> bool {
        if !self.score().is_valid() {
            false
        } else if let Some(ref owner) = self.owner().as_ref() {
            !owner.is_empty()
        } else {
            true
        }
    }

    pub fn is_anonymous(&self) -> bool {
        self.owner().is_none()
    }

    pub fn is_owned(&self) -> bool {
        self.owner().is_some()
    }

    pub fn rating_from_stars(stars: u8, max_stars: u8) -> Score {
        Score(ScoreValue::from(stars.min(max_stars)) / ScoreValue::from(max_stars))
    }

    pub fn star_rating(&self, max_stars: u8) -> u8 {
        ((*self.score() * ScoreValue::from(max_stars)).ceil() as u8).min(max_stars)
    }

    pub fn minmax<'a>(ratings: &[Self], owner: Option<&'a str>) -> Option<(Score, Score)> {
        let count = ratings
            .iter()
            .filter(|rating| {
                owner.is_none()
                    || rating.owner().is_none()
                    || rating.owner().as_ref().map(|owner| owner.as_str()) == owner
            })
            .count();
        if count > 0 {
            let (mut score_min, mut score_max) = (*Score::MAX, *Score::MIN);
            ratings
                .iter()
                .filter(|rating| {
                    owner.is_none()
                        || rating.owner().is_none()
                        || rating.owner().as_ref().map(|owner| owner.as_str()) == owner
                })
                .for_each(|rating| {
                    score_min = score_min.min(*rating.score());
                    score_max = score_max.max(*rating.score());
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
pub struct Comment(/*text*/ String, /*owner*/ Option<String>);

impl Comment {
    pub fn new<T: Into<String>, O: Into<String>>(text: T, owner: Option<O>) -> Self {
        Comment(text.into(), owner.map(O::into))
    }

    pub fn new_anonymous<T: Into<String>>(text: T) -> Self {
        Comment(text.into(), None)
    }

    pub fn new_owned<T: Into<String>, O: Into<String>>(text: T, owner: O) -> Self {
        Comment(text.into(), Some(owner.into()))
    }

    pub fn text(&self) -> &String {
        &self.0
    }

    pub fn owner(&self) -> &Option<String> {
        &self.1
    }

    pub fn is_valid(&self) -> bool {
        if let Some(ref owner) = self.owner().as_ref() {
            !owner.is_empty()
        } else {
            true
        }
    }

    pub fn is_anonymous(&self) -> bool {
        self.owner().is_none()
    }

    pub fn is_owned(&self) -> bool {
        self.owner().is_some()
    }
}
