// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#[test]
fn validate_time_sig() {
    assert!(TimeSignature::new(0, Some(0)).validate().is_err());
    assert!(TimeSignature::new(0, Some(1)).validate().is_err());
    assert!(TimeSignature::new(1, Some(0)).validate().is_err());
    assert!(TimeSignature::new(1, None).validate().is_ok());
    assert!(TimeSignature::new(1, Some(1)).validate().is_ok());
    assert!(TimeSignature::new(3, Some(4)).validate().is_ok());
    assert!(TimeSignature::new(4, Some(4)).validate().is_ok());
    assert!(TimeSignature::new(4, Some(3)).validate().is_ok());
}
