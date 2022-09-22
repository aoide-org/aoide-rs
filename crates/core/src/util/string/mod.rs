use std::borrow::Cow;

// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
    (!from.is_empty()).then_some(from)
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
