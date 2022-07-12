// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    media::content::resolver::{
        ContentPathResolver as _, FileUrlResolver, VirtualFilePathResolver,
    },
    track::Entity,
};

use aoide_repo::track::RecordHeader;

use super::*;

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
