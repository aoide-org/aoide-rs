// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    media::content::resolver::{ContentPathResolver as _, VirtualFilePathResolver},
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
        debug_assert!(record.body.content_url.is_none());
        record.body.content_url = self
            .content_path_resolver
            .resolve_url_from_content_path(path)
            .map_err(|err| {
                log::error!("Failed to convert media source path '{path}' to URL: {err}");
            })
            .ok();
        self.collector.collect(header, record);
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
