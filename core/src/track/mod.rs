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

///////////////////////////////////////////////////////////////////////

use super::*;

pub mod album;
pub mod extra;
pub mod index;
pub mod marker;
pub mod music;
pub mod release;
pub mod tag;

use self::{album::*, extra::*, index::*, marker::*, music::*, release::*};

use crate::{actor::*, media, tag::*, title::*};

use chrono::{
    DateTime as ChronoDateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, ParseError,
    SecondsFormat,
};
use std::str::FromStr;

///////////////////////////////////////////////////////////////////////
// TrackLock
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    // TODO: Move into track collection entry with a single media source
    // for each track in a collection
    pub media_sources: Vec<media::Source>,

    pub musical_signature: MusicalSignature,

    pub release: Release,

    pub album: Album,

    pub titles: Vec<Title>,

    pub actors: Vec<Actor>,

    pub indexes: Indexes,

    pub markers: Markers,

    pub tags: Tags,

    pub extra: Extra,
}

impl Track {
    pub fn purge_media_source_by_uri(&mut self, uri: &str) -> usize {
        let len_before = self.media_sources.len();
        self.media_sources
            .retain(|media_source| media_source.uri != uri);
        debug_assert!(self.media_sources.len() <= len_before);
        len_before - self.media_sources.len()
    }

    pub fn purge_media_source_by_uri_prefix(&mut self, uri_prefix: &str) -> usize {
        let len_before = self.media_sources.len();
        self.media_sources
            .retain(|media_source| !media_source.uri.starts_with(uri_prefix));
        debug_assert!(self.media_sources.len() <= len_before);
        len_before - self.media_sources.len()
    }

    pub fn relocate_media_source_by_uri(&mut self, old_uri: &str, new_uri: &str) -> usize {
        let mut relocated = 0;
        for mut media_source in &mut self.media_sources {
            if media_source.uri == old_uri {
                media_source.uri = new_uri.to_owned();
                relocated += 1;
            }
        }
        relocated
    }

    pub fn relocate_media_source_by_uri_prefix(
        &mut self,
        old_uri_prefix: &str,
        new_uri_prefix: &str,
    ) -> usize {
        let mut relocated = 0;
        for mut media_source in &mut self.media_sources {
            if media_source.uri.starts_with(old_uri_prefix) {
                let mut new_uri = String::with_capacity(
                    new_uri_prefix.len() + (media_source.uri.len() - old_uri_prefix.len()),
                );
                new_uri.push_str(new_uri_prefix);
                new_uri.push_str(&media_source.uri[old_uri_prefix.len()..]);
                media_source.uri = new_uri;
                relocated += 1;
            }
        }
        relocated
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TrackInvalidity {
    MediaSources(media::SourcesInvalidity),
    MusicalSignature(MusicalSignatureInvalidity),
    Release(ReleaseInvalidity),
    Album(AlbumInvalidity),
    Titles(TitlesInvalidity),
    Actors(ActorsInvalidity),
    Indexes(IndexesInvalidity),
    Markers(MarkersInvalidity),
    Tags(TagsInvalidity),
    Extra(ExtraInvalidity),
}

impl Validate for Track {
    type Invalidity = TrackInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.musical_signature, TrackInvalidity::MusicalSignature)
            .merge_result_with(
                media::Sources::validate(self.media_sources.iter()),
                TrackInvalidity::MediaSources,
            )
            .merge_result_with(
                Titles::validate(self.titles.iter()),
                TrackInvalidity::Titles,
            )
            .merge_result_with(
                Actors::validate(self.actors.iter()),
                TrackInvalidity::Actors,
            )
            .validate_with(&self.album, TrackInvalidity::Album)
            .validate_with(&self.release, TrackInvalidity::Release)
            .validate_with(&self.indexes, TrackInvalidity::Indexes)
            .validate_with(&self.markers, TrackInvalidity::Markers)
            .validate_with(&self.tags, TrackInvalidity::Tags)
            .validate_with(&self.extra, TrackInvalidity::Extra)
            .into()
    }
}

pub type Entity = crate::entity::Entity<TrackInvalidity, Track>;

///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DateTime(ChronoDateTime<FixedOffset>);

impl From<ChronoDateTime<FixedOffset>> for DateTime {
    fn from(from: ChronoDateTime<FixedOffset>) -> Self {
        Self(from)
    }
}

impl From<DateTime> for ChronoDateTime<FixedOffset> {
    fn from(from: DateTime) -> Self {
        from.0
    }
}

impl FromStr for DateTime {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl ToString for DateTime {
    fn to_string(&self) -> String {
        self.0.to_rfc3339_opts(SecondsFormat::Secs, true)
    }
}

// 4-digit year
pub type YearType = i16;

// 2-digit month
pub type MonthType = i8;

// 2-digit day of month
pub type DayOfMonthType = i8;

pub const YEAR_MIN: YearType = 1;
pub const YEAR_MAX: YearType = 9999;

pub type YYYYMMDD = i32;

// 8-digit year+month+day (YYYYMMDD)
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Date(YYYYMMDD);

impl Date {
    pub const fn min() -> Self {
        Self(10_000)
    }

    pub const fn max() -> Self {
        Self(99_999_999)
    }

    pub const fn new(val: YYYYMMDD) -> Self {
        Self(val)
    }

    pub fn year(self) -> YearType {
        (self.0 / 10_000) as YearType
    }

    pub fn month(self) -> MonthType {
        ((self.0 % 10_000) / 100) as MonthType
    }

    pub fn day_of_month(self) -> DayOfMonthType {
        (self.0 % 100) as DayOfMonthType
    }

    pub fn from_year(year: YearType) -> Self {
        Self(YYYYMMDD::from(year) * 10_000)
    }

    pub fn is_year(self) -> bool {
        Self::from_year(self.year()) == self
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DateInvalidity {
    Min,
    Max,
    MonthOutOfRange,
    DayOfMonthOutOfRange,
    DayWithoutMonth,
    Invalid,
}

impl Validate for Date {
    type Invalidity = DateInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(*self < Self::min(), DateInvalidity::Min)
            .invalidate_if(*self > Self::max(), DateInvalidity::Min)
            .invalidate_if(
                self.month() < 0 || self.month() > 12,
                DateInvalidity::MonthOutOfRange,
            )
            .invalidate_if(
                self.day_of_month() < 0 || self.day_of_month() > 31,
                DateInvalidity::DayOfMonthOutOfRange,
            )
            .invalidate_if(
                self.month() < 1 && self.day_of_month() > 0,
                DateInvalidity::DayWithoutMonth,
            )
            .invalidate_if(
                self.month() > 0
                    && self.day_of_month() > 0
                    && NaiveDate::from_ymd_opt(
                        i32::from(self.year()),
                        self.month() as u32,
                        self.day_of_month() as u32,
                    )
                    .is_none(),
                DateInvalidity::Invalid,
            )
            .into()
    }
}

impl From<NaiveDateTime> for Date {
    fn from(from: NaiveDateTime) -> Self {
        Self(
            from.year() as YYYYMMDD * 10_000
                + from.month() as YYYYMMDD * 100
                + from.day() as YYYYMMDD,
        )
    }
}

impl From<DateTime> for Date {
    fn from(from: DateTime) -> Self {
        from.0.naive_local().into()
    }
}

impl From<Date> for YYYYMMDD {
    fn from(from: Date) -> Self {
        from.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DateOrDateTime {
    Date(Date),
    DateTime(DateTime),
}

impl From<DateOrDateTime> for Date {
    fn from(from: DateOrDateTime) -> Self {
        match from {
            DateOrDateTime::Date(date) => date,
            DateOrDateTime::DateTime(dt) => dt.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DateOrDateTimeInvalidity {
    Date(DateInvalidity),
}

impl Validate for DateOrDateTime {
    type Invalidity = DateOrDateTimeInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        match self {
            DateOrDateTime::Date(date) => {
                context.validate_with(date, DateOrDateTimeInvalidity::Date)
            }
            DateOrDateTime::DateTime(_) => context,
        }
        .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
