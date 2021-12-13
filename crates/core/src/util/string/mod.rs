// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

pub fn trim_in_place(s: &mut String) {
    s.truncate(s.trim_end().len());
    let drain_start_len = s.len() - s.trim_start().len();
    drop(s.drain(0..drain_start_len));
}

pub fn into_trimmed(s: impl AsRef<str> + Into<String>) -> String {
    let trimmed = s.as_ref().trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.as_bytes().first() == s.as_ref().as_bytes().first()
        && trimmed.as_bytes().last() == s.as_ref().as_bytes().last()
    {
        debug_assert_eq!(trimmed, s.as_ref());
        s.into()
    } else {
        trimmed.to_owned()
    }
}

pub fn non_empty_from(s: impl AsRef<str> + Into<String>) -> Option<String> {
    if s.as_ref().is_empty() {
        None
    } else {
        Some(s.into())
    }
}

pub fn trimmed_non_empty_from(s: impl AsRef<str> + Into<String>) -> Option<String> {
    non_empty_from(into_trimmed(s))
}

pub fn trimmed_non_empty(mut s: String) -> Option<String> {
    trim_in_place(&mut s);
    non_empty_from(s)
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
