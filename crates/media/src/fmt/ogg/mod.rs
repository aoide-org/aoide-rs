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

use lewton::{header::IdentHeader, inside_ogg::OggStreamReader, OggReadError, VorbisError};
use metaflac::block::PictureType;
use num_traits::FromPrimitive as _;
use semval::IsValid as _;
use triseratops::tag::{TagContainer as SeratoTagContainer, TagFormat as SeratoTagFormat};

use aoide_core::{
    audio::{
        channel::ChannelCount,
        signal::{BitrateBps, SampleRateHz},
        AudioContent,
    },
    media::{ApicType, Artwork, Content, ContentMetadataFlags},
    tag::TagsMap,
    track::{
        actor::ActorRole,
        tag::{FACET_COMMENT, FACET_GENRE, FACET_GROUPING, FACET_ISRC, FACET_MOOD},
        Track,
    },
    util::{Canonical, CanonicalizeInto as _},
};

use crate::{
    io::import::*,
    util::{push_next_actor_role_name, serato, try_ingest_embedded_artwork_image},
    Error, Result,
};

use super::vorbis;

impl vorbis::CommentReader for Vec<(String, String)> {
    fn read_first_value(&self, key: &str) -> Option<&str> {
        self.iter().find_map(|(k, v)| {
            if k.eq_ignore_ascii_case(key) {
                Some(v.as_str())
            } else {
                None
            }
        })
    }
}

impl vorbis::CommentWriter for Vec<(String, String)> {
    fn write_multiple_values(&mut self, key: String, values: Vec<String>) {
        // TODO: Optimize or use a different data structure for writing
        self.remove_all_values(&key);
        self.reserve(self.len() + values.len());
        for value in values {
            self.push((key.clone(), value));
        }
    }
    fn remove_all_values(&mut self, key: &str) {
        self.retain(|(cmp_key, _)| cmp_key != key)
    }
}

fn filter_vorbis_comment_values<'a>(
    vorbis_comments: &'a [(String, String)],
    key: &'a str,
) -> impl Iterator<Item = &'a str> + 'a {
    vorbis_comments.iter().filter_map(move |(k, v)| {
        if k.eq_ignore_ascii_case(key) {
            Some(v.as_str())
        } else {
            None
        }
    })
}

fn map_vorbis_err(err: VorbisError) -> Error {
    match err {
        VorbisError::OggError(OggReadError::ReadError(err)) => Error::Io(err),
        err => Error::Other(anyhow::Error::from(err)),
    }
}

pub fn find_embedded_artwork_image(tag: &Tag) -> Option<(ApicType, String, Vec<u8>)> {
    // https://wiki.xiph.org/index.php/VorbisComment#Cover_art
    // The unofficial COVERART field in a VorbisComment tag is deprecated:
    // https://wiki.xiph.org/VorbisComment#Unofficial_COVERART_field_.28deprecated.29
    let (_, vorbis_comments) = tag;
    let picture_iter_by_type = |picture_type: Option<PictureType>| {
        filter_vorbis_comment_values(vorbis_comments, "METADATA_BLOCK_PICTURE")
            .chain(filter_vorbis_comment_values(vorbis_comments, "COVERART"))
            .filter_map(|base64_data| {
                base64::decode(base64_data)
                    .map_err(|err| {
                        tracing::warn!("Failed to decode base64 encoded picture block: {}", err);
                        err
                    })
                    .ok()
            })
            .filter_map(|decoded| {
                metaflac::block::Picture::from_bytes(&decoded[..])
                    .map_err(|err| {
                        tracing::warn!("Failed to decode FLAC picture block: {}", err);
                        err
                    })
                    .ok()
            })
            .filter(move |picture| {
                if let Some(picture_type) = picture_type {
                    picture.picture_type == picture_type
                } else {
                    true
                }
            })
    };
    // Decoding and discarding the blocks multiple times is inefficient
    // but expected to occur only infrequently. Most files will include
    // just a front cover and nothing else.
    picture_iter_by_type(Some(PictureType::CoverFront))
        .chain(picture_iter_by_type(Some(PictureType::Media)))
        .chain(picture_iter_by_type(Some(PictureType::Leaflet)))
        .chain(picture_iter_by_type(Some(PictureType::Other)))
        // otherwise take the first picture that could be parsed
        .chain(picture_iter_by_type(None))
        .map(|p| {
            (
                ApicType::from_u8(p.picture_type as u8).unwrap_or(ApicType::Other),
                p.mime_type,
                p.data,
            )
        })
        .next()
}

pub type Tag = (IdentHeader, Vec<(String, String)>);

pub fn read_tag_from(reader: &mut impl Reader) -> Result<Tag> {
    OggStreamReader::new(reader)
        .map(|r| (r.ident_hdr, r.comment_hdr.comment_list))
        .map_err(map_vorbis_err)
}

pub fn import_track(
    reader: &mut Box<dyn Reader>,
    config: &ImportTrackConfig,
    track: &mut Track,
) -> Result<()> {
    let tag = read_tag_from(reader).map_err(|err| {
        tracing::warn!(
            "Failed to parse metadata from media source '{}': {}",
            track.media_source.path,
            err
        );
        err
    })?;
    let (ident_hdr, vorbis_comments) = &tag;

    if track
        .media_source
        .content_metadata_flags
        .update(ContentMetadataFlags::RELIABLE)
    {
        let channel_count = ChannelCount(ident_hdr.audio_channels.into());
        let channels = if channel_count.is_valid() {
            Some(channel_count.into())
        } else {
            tracing::warn!("Invalid channel count: {}", channel_count.0);
            None
        };
        let bitrate = BitrateBps::from_inner(ident_hdr.bitrate_nominal.into());
        let bitrate = if bitrate.is_valid() {
            Some(bitrate)
        } else {
            tracing::warn!("Invalid bitrate: {}", bitrate);
            None
        };
        let sample_rate = SampleRateHz::from_inner(ident_hdr.audio_sample_rate.into());
        let sample_rate = if sample_rate.is_valid() {
            Some(sample_rate)
        } else {
            tracing::warn!("Invalid sample rate: {}", sample_rate);
            None
        };
        let loudness = vorbis::import_loudness(vorbis_comments);
        let encoder = vorbis::import_encoder(vorbis_comments).map(Into::into);
        // TODO: The duration is not available from any header!?
        let duration = None;
        let audio_content = AudioContent {
            duration,
            channels,
            sample_rate,
            bitrate,
            loudness,
            encoder,
        };
        track.media_source.content = Content::Audio(audio_content);
    }

    if let Some(tempo_bpm) = vorbis::import_tempo_bpm(vorbis_comments) {
        track.metrics.tempo_bpm = Some(tempo_bpm);
    }

    if let Some(key_signature) = vorbis::import_key_signature(vorbis_comments) {
        track.metrics.key_signature = key_signature;
    }

    // Track titles
    let track_titles = vorbis::import_track_titles(vorbis_comments);
    if !track_titles.is_empty() {
        track.titles = Canonical::tie(track_titles);
    }

    // Track actors
    let mut track_actors = Vec::with_capacity(8);
    for name in filter_vorbis_comment_values(vorbis_comments, "ARTIST") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Artist, name.to_owned());
    }
    for name in filter_vorbis_comment_values(vorbis_comments, "ARRANGER") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Arranger, name.to_owned());
    }
    for name in filter_vorbis_comment_values(vorbis_comments, "COMPOSER") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Composer, name.to_owned());
    }
    for name in filter_vorbis_comment_values(vorbis_comments, "CONDUCTOR") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Conductor, name.to_owned());
    }
    for name in filter_vorbis_comment_values(vorbis_comments, "PRODUCER") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Producer, name.to_owned());
    }
    for name in filter_vorbis_comment_values(vorbis_comments, "REMIXER") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Remixer, name.to_owned());
    }
    for name in filter_vorbis_comment_values(vorbis_comments, "MIXER") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Mixer, name.to_owned());
    }
    for name in filter_vorbis_comment_values(vorbis_comments, "DJMIXER") {
        push_next_actor_role_name(&mut track_actors, ActorRole::DjMixer, name.to_owned());
    }
    for name in filter_vorbis_comment_values(vorbis_comments, "ENGINEER") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Engineer, name.to_owned());
    }
    for name in filter_vorbis_comment_values(vorbis_comments, "DIRECTOR") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Director, name.to_owned());
    }
    for name in filter_vorbis_comment_values(vorbis_comments, "LYRICIST") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Lyricist, name.to_owned());
    }
    for name in filter_vorbis_comment_values(vorbis_comments, "WRITER") {
        push_next_actor_role_name(&mut track_actors, ActorRole::Writer, name.to_owned());
    }
    let track_actors = track_actors.canonicalize_into();
    if !track_actors.is_empty() {
        track.actors = Canonical::tie(track_actors);
    }

    let mut album = track.album.untie_replace(Default::default());

    // Album titles
    let album_titles = vorbis::import_album_titles(vorbis_comments);
    if !album_titles.is_empty() {
        album.titles = Canonical::tie(album_titles);
    }

    // Album actors
    let mut album_actors = Vec::with_capacity(4);
    for name in filter_vorbis_comment_values(vorbis_comments, "ALBUMARTIST")
        .chain(filter_vorbis_comment_values(
            vorbis_comments,
            "ALBUM_ARTIST",
        ))
        .chain(filter_vorbis_comment_values(
            vorbis_comments,
            "ALBUM ARTIST",
        ))
        .chain(filter_vorbis_comment_values(vorbis_comments, "ENSEMBLE"))
    {
        push_next_actor_role_name(&mut album_actors, ActorRole::Artist, name.to_owned());
    }
    let album_actors = album_actors.canonicalize_into();
    if !album_actors.is_empty() {
        album.actors = Canonical::tie(album_actors);
    }

    // Album properties
    if let Some(album_kind) = vorbis::import_album_kind(vorbis_comments) {
        album.kind = album_kind;
    }

    track.album = Canonical::tie(album);

    // Release properties
    if let Some(released_at) = vorbis::import_released_at(vorbis_comments) {
        track.release.released_at = Some(released_at);
    }
    if let Some(released_by) = vorbis::import_released_by(vorbis_comments) {
        track.release.released_by = Some(released_by);
    }
    if let Some(copyright) = vorbis::import_release_copyright(vorbis_comments) {
        track.release.copyright = Some(copyright);
    }

    let mut tags_map = TagsMap::default();
    if config.flags.contains(ImportTrackFlags::AOIDE_TAGS) {
        // Pre-populate tags
        if let Some(tags) = vorbis::import_aoide_tags(vorbis_comments) {
            debug_assert_eq!(0, tags_map.total_count());
            tags_map = tags.into();
        }
    }

    // Comment tag
    // The original specification only defines a "DESCRIPTION" field,
    // while MusicBrainz recommends to use "COMMENT".
    // http://www.xiph.org/vorbis/doc/v-comment.html
    // https://picard.musicbrainz.org/docs/mappings
    {
        vorbis::import_faceted_text_tags(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_COMMENT,
            filter_vorbis_comment_values(vorbis_comments, "COMMENT")
                .chain(filter_vorbis_comment_values(vorbis_comments, "DESCRIPTION")),
        );
    }

    // Genre tags
    vorbis::import_faceted_text_tags(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_GENRE,
        filter_vorbis_comment_values(vorbis_comments, "GENRE"),
    );

    // Mood tags
    vorbis::import_faceted_text_tags(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_MOOD,
        filter_vorbis_comment_values(vorbis_comments, "MOOD"),
    );

    // Grouping tags
    vorbis::import_faceted_text_tags(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_GROUPING,
        filter_vorbis_comment_values(vorbis_comments, "GROUPING"),
    );

    // ISRC tags
    vorbis::import_faceted_text_tags(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ISRC,
        filter_vorbis_comment_values(vorbis_comments, "ISRC"),
    );

    if let Some(index) = vorbis::import_track_index(vorbis_comments) {
        track.indexes.track = index;
    }
    if let Some(index) = vorbis::import_disc_index(vorbis_comments) {
        track.indexes.disc = index;
    }
    if let Some(index) = vorbis::import_movement_index(vorbis_comments) {
        track.indexes.movement = index;
    }

    if config.flags.contains(ImportTrackFlags::EMBEDDED_ARTWORK) {
        let artwork =
            if let Some((apic_type, media_type, image_data)) = find_embedded_artwork_image(&tag) {
                try_ingest_embedded_artwork_image(
                    &track.media_source.path,
                    apic_type,
                    &image_data,
                    None,
                    Some(media_type),
                    &mut config.flags.new_artwork_digest(),
                )
                .0
            } else {
                Artwork::Missing
            };
        track.media_source.artwork = Some(artwork);
    }

    // Serato Tags
    if config.flags.contains(ImportTrackFlags::SERATO_MARKERS) {
        let mut serato_tags = SeratoTagContainer::new();
        vorbis::import_serato_markers2(vorbis_comments, &mut serato_tags, SeratoTagFormat::Ogg);

        let track_cues = serato::read_cues(&serato_tags)?;
        if !track_cues.is_empty() {
            track.cues = Canonical::tie(track_cues);
        }

        track.color = serato::read_track_color(&serato_tags);
    }

    Ok(())
}
