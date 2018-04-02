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

pub struct Collection {
    pub uid: String,
    pub name: String,
}

impl Collection {
    fn generate_uid() -> String {
        "TODO: Generate uid".to_string()
    }

    pub fn new<S: Into<String>>(name: S) -> Self {
        let uid = Self::generate_uid();
        Self { uid, name: name.into() }
    }
}
