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

use std::path::Path;

use bitflags::bitflags;

use aoide_core::track::{
    actor::{Actor, ActorKind, ActorRole, Actors},
    Track,
};

use crate::{
    fmt::{flac, mp3, mp4},
    util::tag::FacetedTagMappingConfig,
    Error, Result,
};

use super::import::ImportTrackFlags;

bitflags! {
    pub struct ExportTrackFlags: u16 {
        const ITUNES_ID3V2_GROUPING_MOVEMENT_WORK = ImportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK.bits();
        const AOIDE_TAGS = ImportTrackFlags::AOIDE_TAGS.bits();
        const SERATO_MARKERS = ImportTrackFlags::SERATO_MARKERS.bits();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportTrackConfig {
    pub faceted_tag_mapping: FacetedTagMappingConfig,
    pub flags: ExportTrackFlags,
}

pub fn export_track_to_path(
    path: &Path,
    config: &ExportTrackConfig,
    track: &mut Track,
) -> Result<bool> {
    match track.media_source.content_type.essence_str() {
        "audio/flac" => flac::export_track_to_path(path, config, track),
        "audio/mpeg" => mp3::export_track_to_path(path, config, track),
        "audio/m4a" | "video/mp4" => mp4::export_track_to_path(path, config, track),
        // TODO: Add support for audio/ogg
        _ => Err(Error::UnsupportedContentType(
            track.media_source.content_type.to_owned(),
        )),
    }
}

#[derive(Debug, Clone)]
pub enum FilteredActorNames<'a> {
    Summary(&'a str),
    Primary(Vec<&'a str>), // TODO: Replace with impl Iterator<Item = &'a str>! How?
}

impl<'a> FilteredActorNames<'a> {
    pub fn new(actors: impl IntoIterator<Item = &'a Actor> + Clone, role: ActorRole) -> Self {
        // At most a single summary actor
        debug_assert!(
            Actors::filter_kind_role(actors.clone(), ActorKind::Summary, role).count() <= 1
        );
        // Either a summary actor or primary actors but not both at the same time
        debug_assert!(
            Actors::filter_kind_role(actors.clone(), ActorKind::Summary, role)
                .next()
                .is_none()
                || Actors::filter_kind_role(actors.clone(), ActorKind::Primary, role)
                    .next()
                    .is_none()
        );
        // Secondary actors are not supported yet
        debug_assert!(
            Actors::filter_kind_role(actors.clone(), ActorKind::Secondary, role)
                .next()
                .is_none()
        );
        if let Some(summary_actor) =
            Actors::filter_kind_role(actors.clone(), ActorKind::Summary, role).next()
        {
            Self::Summary(summary_actor.name.as_str())
        } else {
            let primary_actors = Actors::filter_kind_role(actors, ActorKind::Primary, role);
            Self::Primary(primary_actors.map(|actor| actor.name.as_str()).collect())
        }
    }
}