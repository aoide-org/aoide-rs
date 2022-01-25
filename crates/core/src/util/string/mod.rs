use std::borrow::Cow;

// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

pub fn trim_owned_in_place(owned: &mut String) {
    owned.truncate(owned.trim_end().len());
    let drain_start_len = owned.len() - owned.trim_start().len();
    drop(owned.drain(0..drain_start_len));
}

pub fn trim_from_borrowed<'a>(borrowed: impl Into<&'a str>) -> Cow<'a, str> {
    let borrowed = borrowed.into();
    let trimmed = borrowed.trim();
    if trimmed.is_empty() {
        return Cow::Borrowed("");
    }
    if trimmed.as_bytes().first() != borrowed.as_bytes().first()
        || trimmed.as_bytes().last() != borrowed.as_bytes().last()
    {
        return Cow::Borrowed(trimmed);
    }
    debug_assert_eq!(trimmed, borrowed);
    Cow::Borrowed(borrowed)
}

pub fn trim_from<'a>(from: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
    match from.into() {
        Cow::Borrowed(borrowed) => trim_from_borrowed(borrowed),
        Cow::Owned(mut owned) => {
            trim_owned_in_place(&mut owned);
            Cow::Owned(owned)
        }
    }
}

pub fn non_empty_from<'a>(from: impl Into<Cow<'a, str>>) -> Option<Cow<'a, str>> {
    let from = from.into();
    (!from.is_empty()).then(|| from)
}

pub fn trimmed_non_empty_from<'a>(from: impl Into<Cow<'a, str>>) -> Option<Cow<'a, str>> {
    non_empty_from(trim_from(from))
}

#[must_use]
pub fn trimmed_non_empty_from_owned<'a>(mut owned: String) -> Option<Cow<'a, str>> {
    trim_owned_in_place(&mut owned);
    non_empty_from(owned)
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
