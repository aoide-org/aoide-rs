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

/// Predicates for matching URI strings (case-sensitive)
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UriPredicate {
    Prefix(String),
    Exact(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UriRelocation {
    pub predicate: UriPredicate,
    pub replacement: String,
}
