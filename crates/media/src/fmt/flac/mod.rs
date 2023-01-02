// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, path::Path, time::Duration};

use metaflac::block::PictureType;
use num_traits::FromPrimitive as _;

use aoide_core::{
    audio::{channel::ChannelCount, signal::SampleRateHz},
    media::{
        artwork::{ApicType, Artwork},
        content::{AudioContentMetadata, ContentMetadata, ContentMetadataFlags},
    },
    tag::TagsMap,
    track::{
        actor::Role as ActorRole,
        tag::{FACET_ID_COMMENT, FACET_ID_GENRE, FACET_ID_GROUPING, FACET_ID_ISRC, FACET_ID_MOOD},
        Track,
    },
    util::canonical::Canonical,
};

use crate::{
    fmt::vorbis::{
        ALBUM_ARTIST_KEY, ALBUM_ARTIST_KEY2, ALBUM_ARTIST_KEY3, ALBUM_ARTIST_KEY4, ARRANGER_KEY,
        ARTIST_KEY, COMMENT_KEY, COMMENT_KEY2, COMPOSER_KEY, CONDUCTOR_KEY, DIRECTOR_KEY,
        DJMIXER_KEY, ENGINEER_KEY, GENRE_KEY, GROUPING_KEY, ISRC_KEY, LYRICIST_KEY, MIXER_KEY,
        MOOD_KEY, REMIXER_KEY, REMIXER_KEY2, WRITER_KEY,
    },
    io::{
        export::ExportTrackConfig,
        import::{ImportTrackConfig, ImportTrackFlags, Importer, Reader, TrackScope},
    },
    util::{push_next_actor_role_name_from, try_ingest_embedded_artwork_image},
    Error, Result,
};

use super::vorbis;

impl vorbis::CommentReader for metaflac::Tag {
    fn read_first_value(&self, key: &str) -> Option<&str> {
        self.get_vorbis(key).and_then(|mut i| i.next())
    }

    fn filter_values(&self, key: &str) -> Option<Vec<&str>> {
        self.get_vorbis(key).map(|iter| iter.collect())
    }
}

impl vorbis::CommentWriter for metaflac::Tag {
    fn overwrite_single_value(&mut self, key: Cow<'_, str>, value: &'_ str) {
        if self.get_vorbis(&key).is_some() {
            self.write_single_value(key, value.into());
        }
    }
    fn write_multiple_values(&mut self, key: Cow<'_, str>, values: Vec<String>) {
        if values.is_empty() {
            self.remove_vorbis(&key);
        } else {
            self.set_vorbis(key, values);
        }
    }
    fn remove_all_values(&mut self, key: &str) {
        self.remove_vorbis(key);
    }
}

fn map_metaflac_err(err: metaflac::Error) -> Error {
    let metaflac::Error { kind, description } = err;
    match kind {
        metaflac::ErrorKind::Io(err) => Error::Io(err),
        kind => Error::Other(anyhow::Error::from(metaflac::Error { kind, description })),
    }
}

#[must_use]
pub fn find_embedded_artwork_image(tag: &metaflac::Tag) -> Option<(ApicType, &str, &[u8])> {
    tag.pictures()
        .filter_map(|p| {
            if p.picture_type == PictureType::CoverFront {
                Some((ApicType::CoverFront, p))
            } else {
                None
            }
        })
        .chain(tag.pictures().filter_map(|p| {
            if p.picture_type == PictureType::Media {
                Some((ApicType::Media, p))
            } else {
                None
            }
        }))
        .chain(tag.pictures().filter_map(|p| {
            if p.picture_type == PictureType::Leaflet {
                Some((ApicType::Leaflet, p))
            } else {
                None
            }
        }))
        .chain(tag.pictures().filter_map(|p| {
            if p.picture_type == PictureType::Other {
                Some((ApicType::Other, p))
            } else {
                None
            }
        }))
        // otherwise take the first picture that could be parsed
        .chain(tag.pictures().map(|p| {
            (
                ApicType::from_u8(p.picture_type as u8).unwrap_or(ApicType::Other),
                p,
            )
        }))
        .map(|(apic_type, p)| (apic_type, p.mime_type.as_str(), p.data.as_slice()))
        .next()
}

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Metadata(metaflac::Tag);

impl Metadata {
    pub fn read_from(reader: &mut impl Reader) -> Result<Self> {
        metaflac::Tag::read_from(reader)
            .map(Self)
            .map_err(map_metaflac_err)
    }

    #[must_use]
    pub fn find_embedded_artwork_image(&self) -> Option<(ApicType, &str, &[u8])> {
        let Self(metaflac_tag) = self;
        self::find_embedded_artwork_image(metaflac_tag)
    }

    #[must_use]
    pub fn import_audio_content(&self, importer: &mut Importer) -> Option<AudioContentMetadata> {
        let Self(metaflac_tag) = self;
        metaflac_tag.get_streaminfo().map(|streaminfo| {
            let channels = Some(ChannelCount(streaminfo.num_channels.into()).into());
            let duration;
            let sample_rate;
            if streaminfo.sample_rate > 0 {
                duration = Some(
                    Duration::from_secs_f64(
                        streaminfo.total_samples as f64 / streaminfo.sample_rate as f64,
                    )
                    .into(),
                );
                sample_rate = Some(SampleRateHz::from_inner(streaminfo.sample_rate.into()));
            } else {
                duration = None;
                sample_rate = None;
            };
            let loudness = vorbis::import_loudness(importer, metaflac_tag);
            let encoder = vorbis::import_encoder(metaflac_tag).map(Into::into);
            AudioContentMetadata {
                duration,
                channels,
                sample_rate,
                bitrate: None,
                loudness,
                encoder,
            }
        })
    }

    pub fn import_into_track(
        self,
        importer: &mut Importer,
        config: &ImportTrackConfig,
        track: &mut Track,
    ) -> Result<()> {
        if track
            .media_source
            .content
            .metadata_flags
            .update(ContentMetadataFlags::RELIABLE)
        {
            if let Some(audio_content) = self.import_audio_content(importer) {
                track.media_source.content.metadata = ContentMetadata::Audio(audio_content);
            }
        }

        let Self(metaflac_tag) = &self;

        track.metrics.tempo_bpm = vorbis::import_tempo_bpm(importer, metaflac_tag);

        track.metrics.key_signature = vorbis::import_key_signature(importer, metaflac_tag);

        track.titles = vorbis::import_track_titles(importer, metaflac_tag);

        // Track actors
        let mut track_actors = Vec::with_capacity(8);
        if let Some(artists) = metaflac_tag.get_vorbis(ARTIST_KEY) {
            for name in artists {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::Artist, name);
            }
        }
        if let Some(artists) = metaflac_tag.get_vorbis(ARRANGER_KEY) {
            for name in artists {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::Arranger, name);
            }
        }
        if let Some(compersers) = metaflac_tag.get_vorbis(COMPOSER_KEY) {
            for name in compersers {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::Composer, name);
            }
        }
        if let Some(conductors) = metaflac_tag.get_vorbis(CONDUCTOR_KEY) {
            for name in conductors {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::Conductor, name);
            }
        }
        if let Some(producers) = metaflac_tag.get_vorbis(CONDUCTOR_KEY) {
            for name in producers {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::Producer, name);
            }
        }
        if let Some(remixers) = metaflac_tag.get_vorbis(REMIXER_KEY) {
            for name in remixers {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::Remixer, name);
            }
        } else if let Some(remixers) = metaflac_tag.get_vorbis(REMIXER_KEY2) {
            for name in remixers {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::Remixer, name);
            }
        }
        if let Some(mixers) = metaflac_tag.get_vorbis(MIXER_KEY) {
            for name in mixers {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::MixEngineer, name);
            }
        }
        if let Some(mixers) = metaflac_tag.get_vorbis(DJMIXER_KEY) {
            for name in mixers {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::MixDj, name);
            }
        }
        if let Some(engineers) = metaflac_tag.get_vorbis(ENGINEER_KEY) {
            for name in engineers {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::Engineer, name);
            }
        }
        if let Some(engineers) = metaflac_tag.get_vorbis(DIRECTOR_KEY) {
            for name in engineers {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::Director, name);
            }
        }
        if let Some(engineers) = metaflac_tag.get_vorbis(LYRICIST_KEY) {
            for name in engineers {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::Lyricist, name);
            }
        }
        if let Some(engineers) = metaflac_tag.get_vorbis(WRITER_KEY) {
            for name in engineers {
                push_next_actor_role_name_from(&mut track_actors, ActorRole::Writer, name);
            }
        }
        track.actors = importer.finish_import_of_actors(TrackScope::Track, track_actors);

        let mut album = track.album.untie_replace(Default::default());

        // Album titles
        album.titles = vorbis::import_album_titles(importer, metaflac_tag);

        // Album actors
        let mut album_actors = Vec::with_capacity(4);
        for name in metaflac_tag
            .get_vorbis(ALBUM_ARTIST_KEY)
            .into_iter()
            .flatten()
            .chain(
                metaflac_tag
                    .get_vorbis(ALBUM_ARTIST_KEY2)
                    .into_iter()
                    .flatten(),
            )
            .chain(
                metaflac_tag
                    .get_vorbis(ALBUM_ARTIST_KEY3)
                    .into_iter()
                    .flatten(),
            )
            .chain(
                metaflac_tag
                    .get_vorbis(ALBUM_ARTIST_KEY4)
                    .into_iter()
                    .flatten(),
            )
        {
            push_next_actor_role_name_from(&mut album_actors, ActorRole::Artist, name);
        }
        album.actors = importer.finish_import_of_actors(TrackScope::Album, album_actors);

        // Album properties
        album.kind = vorbis::import_album_kind(importer, metaflac_tag);

        track.album = Canonical::tie(album);

        track.recorded_at = vorbis::import_recorded_at(importer, metaflac_tag);
        track.released_at = vorbis::import_released_at(importer, metaflac_tag);
        track.released_orig_at = vorbis::import_released_orig_at(importer, metaflac_tag);

        track.publisher = vorbis::import_publisher(metaflac_tag);
        track.copyright = vorbis::import_copyright(metaflac_tag);

        let mut tags_map = TagsMap::default();

        // Grouping tags
        vorbis::import_faceted_text_tags(
            importer,
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_GROUPING,
            metaflac_tag.get_vorbis(GROUPING_KEY).into_iter().flatten(),
        );

        // Import gigtags from raw grouping tags before any other tags.
        #[cfg(feature = "gigtag")]
        if config.flags.contains(ImportTrackFlags::GIGTAGS) {
            if let Some(faceted_tags) = tags_map.take_faceted_tags(&FACET_ID_GROUPING) {
                tags_map = crate::util::gigtag::import_from_faceted_tags(faceted_tags);
            }
        }

        // Comment tag
        // The original specification only defines a COMMENT_KEY2 field,
        // while MusicBrainz recommends to use COMMENT_KEY.
        // http://www.xiph.org/vorbis/doc/v-comment.html
        // https://picard.musicbrainz.org/docs/mappings
        vorbis::import_faceted_text_tags(
            importer,
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_COMMENT,
            metaflac_tag
                .get_vorbis(COMMENT_KEY)
                .into_iter()
                .flatten()
                .chain(metaflac_tag.get_vorbis(COMMENT_KEY2).into_iter().flatten()),
        );

        // Genre tags
        vorbis::import_faceted_text_tags(
            importer,
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_GENRE,
            metaflac_tag.get_vorbis(GENRE_KEY).into_iter().flatten(),
        );

        // Mood tags
        vorbis::import_faceted_text_tags(
            importer,
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_MOOD,
            metaflac_tag.get_vorbis(MOOD_KEY).into_iter().flatten(),
        );

        // ISRC tag
        vorbis::import_faceted_text_tags(
            importer,
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_ISRC,
            metaflac_tag.get_vorbis(ISRC_KEY).into_iter().flatten(),
        );

        track.indexes.track =
            vorbis::import_track_index(importer, metaflac_tag).unwrap_or_default();
        track.indexes.disc = vorbis::import_disc_index(importer, metaflac_tag).unwrap_or_default();
        track.indexes.movement =
            vorbis::import_movement_index(importer, metaflac_tag).unwrap_or_default();

        if config
            .flags
            .contains(ImportTrackFlags::METADATA_EMBEDDED_ARTWORK)
        {
            let artwork = if let Some((apic_type, mime_type, image_data)) =
                find_embedded_artwork_image(metaflac_tag)
            {
                let (artwork, _, issues) = try_ingest_embedded_artwork_image(
                    apic_type,
                    image_data,
                    None,
                    Some(mime_type),
                    &mut config.flags.new_artwork_digest(),
                );
                issues
                    .into_iter()
                    .for_each(|message| importer.add_issue(message));
                artwork
            } else {
                Artwork::Missing
            };
            track.media_source.artwork = Some(artwork);
        }

        debug_assert!(track.tags.is_empty());
        track.tags = Canonical::tie(tags_map.into());

        #[cfg(feature = "serator-markers")]
        if config.flags.contains(ImportTrackFlags::SERATO_MARKERS) {
            let mut serato_tags = SeratoTagContainer::new();
            if vorbis::import_serato_markers2(
                importer,
                metaflac_tag,
                &mut serato_tags,
                SeratoTagFormat::FLAC,
            ) {
                track.cues = Canonical::tie(crate::util::serato::import_cues(&serato_tags));
                track.color = crate::util::serato::import_track_color(&serato_tags);
            }
        }

        Ok(())
    }
}

pub fn export_track_to_path(
    path: &Path,
    config: &ExportTrackConfig,
    track: &mut Track,
) -> Result<bool> {
    let mut metaflac_tag = match metaflac::Tag::read_from_path(path) {
        Ok(metaflac_tag) => metaflac_tag,
        Err(err) => {
            let content_path = &track.media_source.content.link.path;
            log::warn!("Failed to parse metadata from media source '{content_path}': {err}");
            return Err(map_metaflac_err(err));
        }
    };

    let vorbis_comments_orig = metaflac_tag.vorbis_comments().map(ToOwned::to_owned);
    vorbis::export_track(config, track, &mut metaflac_tag);

    if metaflac_tag.vorbis_comments() == vorbis_comments_orig.as_ref() {
        // Unmodified
        return Ok(false);
    }
    metaflac_tag.write_to_path(path).map_err(map_metaflac_err)?;
    // Modified
    Ok(true)
}
