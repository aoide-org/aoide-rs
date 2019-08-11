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

pub mod actor;
pub mod title;

use aoide_core::{tag::Tag, validate::Validate as _};

use aoide_serde::tag::{FacetedTag, PlainTag};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tags {
    #[serde(rename = "p", skip_serializing_if = "Vec::is_empty", default)]
    pub plain: Vec<PlainTag>, // no duplicate labels allowed

    #[serde(rename = "f", skip_serializing_if = "Vec::is_empty", default)]
    pub faceted: Vec<FacetedTag>, // no duplicate labels per facet allowed
}

impl Validate for Tags {
    fn validate(&self) -> ValidationResult<()> {
        let mut errors = ValidationErrors::new();
        // TODO: Improve and optimize handling of validation errors
        for plain in &self.plain {
            if Tag::from(plain.clone()).validate().is_err() {
                errors.add("plain", ValidationError::new("invalid tag"));
            }
        }
        for faceted in &self.faceted {
            if Tag::from(faceted.clone()).validate().is_err() {
                errors.add("faceted", ValidationError::new("invalid tag"));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
