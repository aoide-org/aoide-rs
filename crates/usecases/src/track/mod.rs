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

use semval::Validate as _;

use aoide_core::{
    media::content::resolver::{
        ContentPathResolver as _, FileUrlResolver, VirtualFilePathResolver,
    },
    track::*,
};
use aoide_repo::track::RecordHeader;

use super::*;

pub mod find_duplicates;
pub mod find_unsynchronized;
pub mod import_and_replace;
pub mod purge;
pub mod replace;
pub mod resolve;
pub mod search;

#[derive(Debug)]
pub struct ResolveUrlFromVirtualFilePathCollector<'c, C> {
    pub content_path_resolver: VirtualFilePathResolver,
    pub collector: &'c mut C,
}

impl<'c, C> RecordCollector for ResolveUrlFromVirtualFilePathCollector<'c, C>
where
    C: RecordCollector<Header = RecordHeader, Record = Entity>,
{
    type Header = RecordHeader;
    type Record = Entity;

    fn collect(&mut self, header: Self::Header, mut record: Self::Record) {
        let path = &record.body.track.media_source.content_link.path;
        match self
            .content_path_resolver
            .resolve_url_from_content_path(path)
        {
            Ok(url) => {
                record.body.track.media_source.content_link.path = FileUrlResolver
                    .resolve_path_from_url(&url)
                    .expect("percent-encoded URL");
                self.collector.collect(header, record);
            }
            Err(err) => {
                log::error!("Failed to convert media source path '{path}' to URL: {err}");
            }
        }
    }
}

impl<'c, C> ReservableRecordCollector for ResolveUrlFromVirtualFilePathCollector<'c, C>
where
    C: ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
{
    fn reserve(&mut self, additional: usize) {
        self.collector.reserve(additional);
    }
}

#[derive(Debug)]
pub struct ValidatedInput(Track);

pub fn validate_input(track: Track) -> InputResult<(ValidatedInput, Vec<TrackInvalidity>)> {
    // Many tracks are expected to be inconsistent and invalid to some
    // extent and we simply cannot reject all of them. The invalidities
    // are returned together with the validated input.
    let invalidaties = track
        .validate()
        .map_err(|err| err.into_iter().collect())
        .err()
        .unwrap_or_default();
    Ok((ValidatedInput(track), invalidaties))
}
