// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::Seek as _,
    path::Path,
};

use bitflags::bitflags;
use lofty::{
    file::{AudioFile, FileType},
    flac::FlacFile,
    iff::aiff::AiffFile,
    mp4::Mp4File,
    mpeg::MpegFile,
    ogg::{OpusFile, VorbisFile},
    probe::Probe,
};

use aoide_core::{
    media::artwork::EditEmbeddedArtworkImage,
    tag::FacetId,
    track::{
        Track,
        actor::{Actor, Actors, Kind as ActorKind, Role as ActorRole},
    },
};

use crate::{Error, Result, fmt::parse_options, util::tag::FacetedTagMappingConfig};

use super::import::ImportTrackFlags;

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
    let parse_options = parse_options();
    let write_options = Default::default();
    match file_type {
        FileType::Aiff => {
            let mut aiff_file = <AiffFile as AudioFile>::read_from(file, parse_options)?;
            crate::fmt::aiff::export_track_to_file(
                &mut aiff_file,
                config,
                track,
                edit_embedded_artwork_image,
            )?;
            aiff_file.save_to(file, write_options)?;
        }
        FileType::Flac => {
            let mut flac_file = <FlacFile as AudioFile>::read_from(file, parse_options)?;
            crate::fmt::flac::export_track_to_file(
                &mut flac_file,
                config,
                track,
                edit_embedded_artwork_image,
            )?;
            flac_file.save_to(file, write_options)?;
        }
        FileType::Mp4 => {
            let mut mp4_file = <Mp4File as AudioFile>::read_from(file, parse_options)?;
            crate::fmt::mp4::export_track_to_file(
                &mut mp4_file,
                config,
                track,
                edit_embedded_artwork_image,
            )?;
            mp4_file.save_to(file, write_options)?;
        }
        FileType::Mpeg => {
            let mut mpeg_file = <MpegFile as AudioFile>::read_from(file, parse_options)?;
            crate::fmt::mpeg::export_track_to_file(
                &mut mpeg_file,
                config,
                track,
                edit_embedded_artwork_image,
            )?;
            mpeg_file.save_to(file, write_options)?;
        }
        FileType::Opus => {
            let mut opus_file = <OpusFile as AudioFile>::read_from(file, parse_options)?;
            crate::fmt::opus::export_track_to_file(
                &mut opus_file,
                config,
                track,
                edit_embedded_artwork_image,
            )?;
            opus_file.save_to(file, write_options)?;
        }
        FileType::Vorbis => {
            let mut vorbis_file = <VorbisFile as AudioFile>::read_from(file, parse_options)?;
            crate::fmt::ogg::export_track_to_file(
                &mut vorbis_file,
                config,
                track,
                edit_embedded_artwork_image,
            )?;
            vorbis_file.save_to(file, write_options)?;
        }
        _ => {
            log::debug!(
                "Skipping export of track {media_source_content_link:?}: {config:?}",
                media_source_content_link = track.media_source.content.link
            );
            return Err(Error::UnsupportedContentType(
                track.media_source.content.r#type.clone(),
            ));
        }
    }
    Ok(())
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
