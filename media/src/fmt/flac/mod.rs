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

use crate::{
    io::import::{self, *},
    util::{digest::MediaDigest, parse_artwork_from_embedded_image, push_next_actor_role_name},
    Result,
};

use aoide_core::{
    audio::{channel::ChannelCount, signal::SampleRateHz, AudioContent},
    media::{Content, ContentMetadataFlags},
    tag::{Tags, TagsMap},
    track::{
        actor::ActorRole,
        tag::{FACET_CGROUP, FACET_COMMENT, FACET_GENRE, FACET_MOOD},
        Track,
    },
    util::{Canonical, CanonicalizeInto as _},
};

use aoide_core_serde::tag::Tags as SerdeTags;

use metaflac::block::PictureType;
use std::time::Duration;

use super::vorbis;

impl vorbis::CommentReader for metaflac::Tag {
    fn read_first_value(&self, key: &str) -> Option<&str> {
        self.get_vorbis(key).and_then(|mut i| i.next())
    }
}

fn first_vorbis_value<'a>(flac_tag: &'a metaflac::Tag, key: &str) -> Option<&'a str> {
    flac_tag.get_vorbis(key).and_then(|mut i| i.next())
}

#[derive(Debug)]
pub struct ImportTrack;

impl import::ImportTrack for ImportTrack {
    fn import_track(
        &self,
        config: &ImportTrackConfig,
        options: ImportTrackOptions,
        mut track: Track,
        reader: &mut Box<dyn Reader>,
    ) -> Result<Track> {
        let flac_tag = metaflac::Tag::read_from(reader).map_err(anyhow::Error::from)?;

        if track
            .media_source
            .content_metadata_flags
            .update(ContentMetadataFlags::RELIABLE)
        {
            if let Some(streaminfo) = flac_tag.get_streaminfo() {
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
                    sample_rate = Some(SampleRateHz::new(streaminfo.sample_rate.into()));
                } else {
                    duration = None;
                    sample_rate = None;
                };
                let loudness = vorbis::import_loudness(&flac_tag);
                let encoder = vorbis::import_encoder(&flac_tag).map(Into::into);
                let audio_content = AudioContent {
                    duration,
                    channels,
                    sample_rate,
                    bitrate: None,
                    loudness,
                    encoder,
                };
                track.media_source.content = Content::Audio(audio_content);
            }
        }

        if let Some(tempo_bpm) = vorbis::import_tempo_bpm(&flac_tag) {
            track.metrics.tempo_bpm = Some(tempo_bpm);
        }

        if let Some(key_signature) = vorbis::import_key_signature(&flac_tag) {
            track.metrics.key_signature = key_signature;
        }

        // Track titles
        let track_titles = vorbis::import_track_titles(&flac_tag);
        if !track_titles.is_empty() {
            track.titles = Canonical::tie(track_titles);
        }

        // Track actors
        let mut track_actors = Vec::with_capacity(8);
        if let Some(artists) = flac_tag.get_vorbis("ARTIST") {
            for name in artists {
                push_next_actor_role_name(&mut track_actors, ActorRole::Artist, name.to_owned());
            }
        }
        if let Some(artists) = flac_tag.get_vorbis("COMPOSER") {
            for name in artists {
                push_next_actor_role_name(&mut track_actors, ActorRole::Composer, name.to_owned());
            }
        }
        if let Some(artists) = flac_tag.get_vorbis("CONDUCTOR") {
            for name in artists {
                push_next_actor_role_name(&mut track_actors, ActorRole::Conductor, name.to_owned());
            }
        }
        if let Some(artists) = flac_tag.get_vorbis("PRODUCER") {
            for name in artists {
                push_next_actor_role_name(&mut track_actors, ActorRole::Producer, name.to_owned());
            }
        }
        if let Some(artists) = flac_tag.get_vorbis("REMIXER") {
            for name in artists {
                push_next_actor_role_name(&mut track_actors, ActorRole::Remixer, name.to_owned());
            }
        }
        let track_actors = track_actors.canonicalize_into();
        if !track_actors.is_empty() {
            track.actors = Canonical::tie(track_actors);
        }

        let mut album = track.album.untie();

        // Album titles
        let album_titles = vorbis::import_album_titles(&flac_tag);
        if !album_titles.is_empty() {
            album.titles = Canonical::tie(album_titles);
        }

        // Album actors
        let mut album_actors = Vec::with_capacity(4);
        if let Some(artists) = flac_tag
            .get_vorbis("ALBUMARTIST")
            .or_else(|| flac_tag.get_vorbis("ALBUM_ARTIST"))
            .or_else(|| flac_tag.get_vorbis("ALBUM ARTIST"))
            .or_else(|| flac_tag.get_vorbis("ENSEMBLE"))
        {
            for name in artists {
                push_next_actor_role_name(&mut album_actors, ActorRole::Artist, name.to_owned());
            }
        }
        let album_actors = album_actors.canonicalize_into();
        if !album_actors.is_empty() {
            album.actors = Canonical::tie(album_actors);
        }

        // Album properties
        if let Some(album_kind) = vorbis::import_album_kind(&flac_tag) {
            album.kind = album_kind;
        }

        track.album = Canonical::tie(album);

        // Release properties
        if let Some(released_at) = vorbis::import_released_at(&flac_tag) {
            track.release.released_at = Some(released_at);
        }
        if let Some(released_by) = vorbis::import_released_by(&flac_tag) {
            track.release.released_by = Some(released_by);
        }
        if let Some(copyright) = vorbis::import_release_copyright(&flac_tag) {
            track.release.copyright = Some(copyright);
        }

        let mut tags_map = TagsMap::default();
        if options.contains(ImportTrackOptions::MIXXX_CUSTOM_TAGS) {
            if let Some(json) = first_vorbis_value(&flac_tag, "MIXXX_CUSTOM_TAGS") {
                if let Some(custom_tags) = serde_json::from_str::<SerdeTags>(json)
                    .map_err(|err| {
                        log::warn!("Failed to parse Mixxx custom tags: {}", err);
                        err
                    })
                    .ok()
                    .map(Tags::from)
                {
                    // Initialize map with all existing custom tags as starting point
                    debug_assert_eq!(0, tags_map.total_count());
                    tags_map = custom_tags.into();
                }
            }
        }

        // Comment tag
        // The original specification only defines a "DESCRIPTION" field,
        // while MusicBrainz recommends to use "COMMENT". Mixxx follows
        // MusicBrainz.
        // http://www.xiph.org/vorbis/doc/v-comment.html
        // https://picard.musicbrainz.org/docs/mappings
        if let Some(comments) = flac_tag
            .get_vorbis("COMMENT")
            .or_else(|| flac_tag.get_vorbis("DESCRIPTION"))
        {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_COMMENT,
                comments,
            );
        }

        // Genre tags
        if let Some(genres) = flac_tag.get_vorbis("GENRE") {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_GENRE,
                genres,
            );
        }

        // Mood tags
        if let Some(moods) = flac_tag.get_vorbis("MOOD") {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_MOOD,
                moods,
            );
        }

        // Grouping tags
        if let Some(groupings) = flac_tag.get_vorbis("GROUPING") {
            vorbis::import_faceted_text_tags(
                &mut tags_map,
                &config.faceted_tag_mapping,
                &FACET_CGROUP,
                groupings,
            );
        }

        if let Some(index) = vorbis::import_track_index(&flac_tag) {
            track.indexes.track = index;
        }
        if let Some(index) = vorbis::import_disc_index(&flac_tag) {
            track.indexes.disc = index;
        }
        if let Some(index) = vorbis::import_movement_index(&flac_tag) {
            track.indexes.movement = index;
        }

        if options.contains(ImportTrackOptions::ARTWORK) {
            let mut image_digest = if options.contains(ImportTrackOptions::ARTWORK_DIGEST) {
                if options.contains(ImportTrackOptions::ARTWORK_DIGEST_SHA256) {
                    // Compatibility
                    MediaDigest::sha256()
                } else {
                    // Default
                    MediaDigest::new()
                }
            } else {
                Default::default()
            };
            let artwork = flac_tag
                .pictures()
                .filter(|p| p.picture_type == PictureType::CoverFront)
                .chain(
                    flac_tag
                        .pictures()
                        .filter(|p| p.picture_type == PictureType::Media),
                )
                .chain(
                    flac_tag
                        .pictures()
                        .filter(|p| p.picture_type == PictureType::Leaflet),
                )
                .chain(
                    flac_tag
                        .pictures()
                        .filter(|p| p.picture_type == PictureType::Other),
                )
                // otherwise take the first picture that could be parsed
                .chain(flac_tag.pictures())
                .filter_map(|p| parse_artwork_from_embedded_image(&p.data, None, &mut image_digest))
                .next();
            if let Some(artwork) = artwork {
                track.media_source.artwork = artwork;
            }
        }

        debug_assert!(track.tags.is_empty());
        track.tags = Canonical::tie(tags_map.into());

        Ok(track)
    }
}
