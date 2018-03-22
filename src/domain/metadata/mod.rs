use std::fmt;
use std::ops::Deref;

///////////////////////////////////////////////////////////////////////
/// Confidence
///////////////////////////////////////////////////////////////////////

pub type ConfidenceValue = f32;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(pub ConfidenceValue);

impl From<ConfidenceValue> for Confidence {
    fn from(count: ConfidenceValue) -> Self {
        Confidence(count)
    }
}

impl Deref for Confidence {
    type Target = ConfidenceValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<ConfidenceValue> for Confidence {
    fn into(self) -> ConfidenceValue {
        *self
    }
}

impl Confidence {
    pub const MIN: Confidence = Confidence(0f32);
    pub const MAX: Confidence = Confidence(1f32);

    pub fn is_valid(&self) -> bool {
        (*self >= Self::MIN) && (*self <= Self::MAX)
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.1}%", self.0 * 100f32)
    }
}

///////////////////////////////////////////////////////////////////////
/// Tag
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
  #[serde(skip_serializing_if = "Option::is_none")] facet: Option<String>, // lowercase / case-insensitive
  term: String,
  confidence: Confidence,
}

impl Tag {
  pub fn new<T: Into<String>, C: Into<Confidence>>(term: T, confidence: C) -> Self {
    Self {
      facet: None,
      term: term.into(),
      confidence: confidence.into(),
    }
  }

  pub fn new_faceted<T: Into<String>, C: Into<Confidence>>(facet: &str, term: T, confidence: C) -> Self {
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
#[serde(rename_all = "camelCase")]
pub struct Rating {
  #[serde(skip_serializing_if = "Option::is_none")] owner: Option<String>,
  rating: Confidence,
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
}

///////////////////////////////////////////////////////////////////////
/// Comment
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
  #[serde(skip_serializing_if = "Option::is_none")] owner: Option<String>,
  comment: String,
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
}
