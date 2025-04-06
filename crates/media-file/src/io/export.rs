// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::Seek as _,
    path::Path,
};

use bitflags::bitflags;
use lofty::{file::FileType, probe::Probe};

use aoide_core::{
    media::artwork::EditEmbeddedArtworkImage,
    tag::FacetId,
    track::{
        Track,
        actor::{Actor, Actors, Kind as ActorKind, Role as ActorRole},
    },
};

use super::import::ImportTrackFlags;
use crate::{Error, Result, util::tag::FacetedTagMappingConfig};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct ExportTrackFlags: u16 {
        /// See also: [`super::import::ImportTrackFlags`]
        const COMPATIBILITY_ID3V2_APPLE_GRP1 = ImportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1.bits();

        #[cfg(feature = "serato-markers")]
        const SERATO_MARKERS                 = ImportTrackFlags::SERATO_MARKERS.bits();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportTrackConfig {
    pub faceted_tag_mapping: FacetedTagMappingConfig,
    pub flags: ExportTrackFlags,

    // Encode gig tags into the corresponding file tag.
    #[cfg(feature = "gigtag")]
    pub encode_gigtags: Option<FacetId>,
}

impl Default for ExportTrackConfig {
    fn default() -> Self {
        Self {
            faceted_tag_mapping: Default::default(),
            flags: ExportTrackFlags::all()
                .difference(ExportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1),
            #[cfg(feature = "gigtag")]
            encode_gigtags: None,
        }
    }
}

pub fn export_track_to_file_path(
    path: &Path,
    file_ext: Option<&str>,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    let mut file = OpenOptions::new().write(true).open(path)?;
    let file_ext = file_ext.or_else(|| path.extension().and_then(OsStr::to_str));
    export_track_to_file(
        &mut file,
        file_ext,
        config,
        track,
        edit_embedded_artwork_image,
    )
}

pub fn export_track_to_file(
    file: &mut File,
    file_ext: Option<&str>,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    let file_type = if let Some(file_type) = file_ext.and_then(FileType::from_ext) {
        file_type
    } else {
        let probe = Probe::new(file.try_clone()?).guess_file_type()?;
        let Some(file_type) = probe.file_type() else {
            log::debug!(
                "Skipping export of track {media_source_content_link:?}: {config:?}",
                media_source_content_link = track.media_source.content.link
            );
            return Err(Error::UnsupportedContentType(
                track.media_source.content.r#type.clone(),
            ));
        };
        file_type
    };
    // Ensure that the file could be read again
    file.rewind()?;
    match file_type {
        FileType::Aiff => {
            if track.media_source.content.r#type.essence_str() != "audio/aiff" {
                return Err(Error::UnsupportedContentType(
                    track.media_source.content.r#type.clone(),
                ));
            }
            crate::fmt::aiff::export_track_to_file(file, config, track, edit_embedded_artwork_image)
        }
        FileType::Flac => {
            if track.media_source.content.r#type.essence_str() != "audio/flac" {
                return Err(Error::UnsupportedContentType(
                    track.media_source.content.r#type.clone(),
                ));
            }
            crate::fmt::flac::export_track_to_file(file, config, track, edit_embedded_artwork_image)
        }
        FileType::Mp4 => {
            if !matches!(
                track.media_source.content.r#type.essence_str(),
                "audio/m4a" | "audio/mp4" | "video/mp4"
            ) {
                return Err(Error::UnsupportedContentType(
                    track.media_source.content.r#type.clone(),
                ));
            }
            crate::fmt::mp4::export_track_to_file(file, config, track, edit_embedded_artwork_image)
        }
        FileType::Mpeg => {
            if track.media_source.content.r#type.essence_str() != "audio/mpeg" {
                return Err(Error::UnsupportedContentType(
                    track.media_source.content.r#type.clone(),
                ));
            }
            crate::fmt::mpeg::export_track_to_file(file, config, track, edit_embedded_artwork_image)
        }
        FileType::Opus => {
            if track.media_source.content.r#type.essence_str() != "audio/ogg" {
                return Err(Error::UnsupportedContentType(
                    track.media_source.content.r#type.clone(),
                ));
            }
            crate::fmt::ogg::export_track_to_file(file, config, track, edit_embedded_artwork_image)
        }
        FileType::Vorbis => {
            if track.media_source.content.r#type.essence_str() != "audio/opus" {
                return Err(Error::UnsupportedContentType(
                    track.media_source.content.r#type.clone(),
                ));
            }
            crate::fmt::opus::export_track_to_file(file, config, track, edit_embedded_artwork_image)
        }
        _ => {
            log::debug!(
                "Skipping export of track {media_source_content_link:?}: {config:?}",
                media_source_content_link = track.media_source.content.link
            );
            Err(Error::UnsupportedContentType(
                track.media_source.content.r#type.clone(),
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub enum FilteredActorNames<'a> {
    Summary(&'a str),
    Individual(Vec<&'a str>), // TODO: Replace with impl Iterator<Item = &'a str>! How?
    Sorting(&'a str),
}

impl<'a> FilteredActorNames<'a> {
    #[must_use]
    pub fn filter(
        actors: impl IntoIterator<Item = &'a Actor> + Clone,
        role: ActorRole,
        kind: ActorKind,
    ) -> Option<Self> {
        match kind {
            ActorKind::Summary | ActorKind::Individual => {
                // At most a single summary actor
                debug_assert!(
                    Actors::filter_kind_role(actors.clone(), ActorKind::Summary, role).count() <= 1
                );
                // Either a summary actor or individual actors but not both at the same time
                debug_assert!(
                    Actors::filter_kind_role(actors.clone(), ActorKind::Summary, role)
                        .next()
                        .is_none()
                        || Actors::filter_kind_role(actors.clone(), ActorKind::Individual, role)
                            .next()
                            .is_none()
                );
                if let Some(summary_actor) =
                    Actors::filter_kind_role(actors.clone(), ActorKind::Summary, role).next()
                {
                    Some(Self::Summary(summary_actor.name.as_str()))
                } else {
                    let individual_actors =
                        Actors::filter_kind_role(actors, ActorKind::Individual, role)
                            .map(|actor| actor.name.as_str())
                            .collect::<Vec<_>>();
                    if individual_actors.is_empty() {
                        None
                    } else {
                        Some(Self::Individual(individual_actors))
                    }
                }
            }
            ActorKind::Sorting => {
                // At most a single sorting actor
                debug_assert!(
                    Actors::filter_kind_role(actors.clone(), ActorKind::Sorting, role).count() <= 1
                );
                Actors::filter_kind_role(actors, ActorKind::Sorting, role)
                    .next()
                    .map(|actor| Self::Sorting(actor.name.as_str()))
            }
        }
    }
}
