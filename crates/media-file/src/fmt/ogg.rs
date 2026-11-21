// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shared implementation for Ogg containers with Vorbis Comments,
//! i.e. Ogg/Vorbis and Ogg/Opus files.

use lofty::{
    file::TaggedFile,
    ogg::{OpusFile, VorbisFile, tag::VorbisComments},
};

use aoide_core::{media::artwork::EditEmbeddedArtworkImage, track::Track};

use crate::{
    Result,
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, Importer},
    },
};

use super::vorbis::export_track_to_tag;

/// An Ogg file with Vorbis Comments.
pub(crate) trait OggFile: Into<TaggedFile> {
    /// The only accepted content type for exporting metadata.
    const CONTENT_TYPE: &'static str;

    #[cfg(feature = "serato-markers")]
    fn vorbis_comments(&self) -> &VorbisComments;

    fn vorbis_comments_mut(&mut self) -> &mut VorbisComments;
}

impl OggFile for VorbisFile {
    const CONTENT_TYPE: &'static str = "audio/ogg";

    #[cfg(feature = "serato-markers")]
    fn vorbis_comments(&self) -> &VorbisComments {
        Self::vorbis_comments(self)
    }

    fn vorbis_comments_mut(&mut self) -> &mut VorbisComments {
        Self::vorbis_comments_mut(self)
    }
}

impl OggFile for OpusFile {
    const CONTENT_TYPE: &'static str = "audio/opus";

    #[cfg(feature = "serato-markers")]
    fn vorbis_comments(&self) -> &VorbisComments {
        Self::vorbis_comments(self)
    }

    fn vorbis_comments_mut(&mut self) -> &mut VorbisComments {
        Self::vorbis_comments_mut(self)
    }
}

pub(crate) fn import_file_into_track<F: OggFile>(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    ogg_file: F,
    track: &mut Track,
) {
    // Pre-processing
    #[cfg(feature = "serato-markers")]
    let serato_tags = config
        .flags
        .contains(crate::io::import::ImportTrackFlags::SERATO_MARKERS)
        .then(|| {
            super::vorbis::import_serato_markers(
                importer,
                ogg_file.vorbis_comments(),
                triseratops::tag::TagFormat::Ogg,
            )
        })
        .flatten();

    // Generic import
    let tagged_file = ogg_file.into();
    super::import_tagged_file_into_track(importer, config, tagged_file, track);

    // Post-processing
    #[cfg(feature = "serato-markers")]
    if let Some(serato_tags) = &serato_tags {
        super::import_serato_tags(track, serato_tags);
    }
}

pub(crate) fn export_track_to_file<F: OggFile>(
    ogg_file: &mut F,
    config: &ExportTrackConfig,
    track: &mut Track,
    edit_embedded_artwork_image: Option<EditEmbeddedArtworkImage>,
) -> Result<()> {
    super::ensure_content_type(track, &[F::CONTENT_TYPE])?;
    export_track_to_tag(
        ogg_file.vorbis_comments_mut(),
        config,
        track,
        edit_embedded_artwork_image,
    );
    Ok(())
}
