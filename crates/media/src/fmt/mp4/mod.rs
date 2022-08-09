// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{iter::once, path::Path};

use image::ImageFormat;
use mp4ameta::{
    AdvisoryRating as Mp4AdvisoryRating, ChannelConfig, Data, DataIdent, Fourcc, FreeformIdent,
    Ident, ImgFmt, SampleRate as Mp4SampleRate, Tag as Mp4Tag, STANDARD_GENRES,
};

use aoide_core::{
    audio::{
        channel::{ChannelCount, ChannelLayout, Channels},
        signal::{BitrateBps, BitsPerSecond, SampleRateHz, SamplesPerSecond},
    },
    media::{
        artwork::{ApicType, Artwork},
        content::{AudioContentMetadata, ContentMetadata, ContentMetadataFlags},
        AdvisoryRating,
    },
    music::{key::KeySignature, tempo::TempoBpm},
    tag::{FacetedTags, PlainTag, Score as TagScore, TagsMap},
    track::{
        actor::Role as ActorRole,
        album::Kind as AlbumKind,
        metric::MetricsFlags,
        tag::{
            FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING,
            FACET_ID_ISRC, FACET_ID_MOOD, FACET_ID_XID,
        },
        title::{Kind as TitleKind, Titles},
        Track,
    },
    util::{canonical::Canonical, string::trimmed_non_empty_from_owned},
};

use crate::{
    io::{
        export::{ExportTrackConfig, ExportTrackFlags, FilteredActorNames},
        import::{ImportTrackConfig, ImportTrackFlags, Importer, Reader, TrackScope},
    },
    util::{
        format_valid_replay_gain, format_validated_tempo_bpm, ingest_title_from_owned,
        push_next_actor_role_name, tag::TagMappingConfig, try_ingest_embedded_artwork_image,
    },
    Error, Result,
};

fn map_mp4ameta_err(err: mp4ameta::Error) -> Error {
    let mp4ameta::Error { kind, description } = err;
    match kind {
        mp4ameta::ErrorKind::Io(err) => Error::Io(err),
        kind => Error::Other(anyhow::Error::from(mp4ameta::Error { kind, description })),
    }
}

fn read_bitrate(bitrate: u32) -> Option<BitrateBps> {
    let bits_per_second = bitrate as BitsPerSecond;
    let bitrate_bps = BitrateBps::from_inner(bits_per_second);
    if bitrate_bps >= BitrateBps::min() {
        Some(bitrate_bps)
    } else {
        None
    }
}

fn read_channels(channel_config: ChannelConfig) -> Channels {
    use ChannelConfig::*;
    let channels = match channel_config {
        Mono => Channels::Layout(ChannelLayout::Mono),
        Stereo => Channels::Layout(ChannelLayout::Stereo),
        Three => Channels::Count(ChannelCount(3)),
        Four => Channels::Count(ChannelCount(4)),
        Five => Channels::Count(ChannelCount(5)),
        FiveOne => Channels::Layout(ChannelLayout::FiveOne),
        SevenOne => Channels::Layout(ChannelLayout::SevenOne),
    };
    // Discard the layout and only return the channel count.
    // In the database only the channel count will be stored.
    // Otherwise imported metadata would repeatedly be detected
    // as modified!
    channels.count().into()
}

const COM_APPLE_ITUNES_FREEFORM_MEAN: &str = "com.apple.iTunes";

const IDENT_ALBUM_ARTIST: Fourcc = Fourcc(*b"aART");

const IDENT_ARTIST: Fourcc = Fourcc(*b"\xA9ART");

const IDENT_COMMENT: Fourcc = Fourcc(*b"\xA9cmt");

const IDENT_COMPOSER: Fourcc = Fourcc(*b"\xA9wrt");

const IDENT_DESCRIPTION: Fourcc = Fourcc(*b"desc");

const IDENT_DIRECTOR: Fourcc = Fourcc(*b"\xA9dir");

const IDENT_GENRE: Fourcc = Fourcc(*b"\xA9gen");

const IDENT_GROUPING: Fourcc = Fourcc(*b"\xA9grp");

const IDENT_YEAR: Fourcc = Fourcc(*b"\xA9day");

const IDENT_XID: Fourcc = Fourcc(*b"xid ");

const IDENT_BPM: FreeformIdent<'static> = FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "BPM");

const IDENT_INITIAL_KEY: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "initialkey");
const KEY_IDENT: FreeformIdent<'static> = FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "KEY");

const IDENT_REPLAYGAIN_TRACK_GAIN: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "replaygain_track_gain");

const IDENT_SUBTITLE: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "SUBTITLE");

const IDENT_CONDUCTOR: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "CONDUCTOR");

const IDENT_ENGINEER: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "ENGINEER");

const IDENT_LYRICIST: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "LYRICIST");

const IDENT_MIXER: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "MIXER");

const IDENT_PRODUCER: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "PRODUCER");

const IDENT_REMIXER: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "REMIXER");

const IDENT_LABEL: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "LABEL");

const IDENT_MOOD: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "MOOD");

const IDENT_ISRC: FreeformIdent<'static> =
    FreeformIdent::new(COM_APPLE_ITUNES_FREEFORM_MEAN, "ISRC");

#[cfg(feature = "serato-markers")]
const SERATO_MARKERS_IDENT: FreeformIdent<'static> = FreeformIdent::new(
    <triseratops::tag::Markers as triseratops::tag::format::mp4::MP4Tag>::MP4_ATOM_FREEFORM_MEAN,
    <triseratops::tag::Markers as triseratops::tag::format::mp4::MP4Tag>::MP4_ATOM_FREEFORM_NAME,
);

#[cfg(feature = "serato-markers")]
const SERATO_MARKERS2_IDENT: FreeformIdent<'static> = FreeformIdent::new(
    <triseratops::tag::Markers2 as triseratops::tag::format::mp4::MP4Tag>::MP4_ATOM_FREEFORM_MEAN,
    <triseratops::tag::Markers2 as triseratops::tag::format::mp4::MP4Tag>::MP4_ATOM_FREEFORM_NAME,
);

fn find_embedded_artwork_image(tag: &Mp4Tag) -> Option<(ApicType, ImageFormat, &[u8])> {
    tag.artworks()
        .map(|img| {
            let image_format = match img.fmt {
                ImgFmt::Jpeg => ImageFormat::Jpeg,
                ImgFmt::Png => ImageFormat::Png,
                ImgFmt::Bmp => ImageFormat::Bmp,
            };
            // Unspecific APIC type
            (ApicType::Other, image_format, img.data)
        })
        // Only consider the 1st embedded image as artwork
        .next()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata(Mp4Tag);

impl Metadata {
    pub fn read_from(reader: &mut impl Reader) -> Result<Self> {
        Mp4Tag::read_from(reader)
            .map(Self)
            .map_err(map_mp4ameta_err)
    }

    #[must_use]
    pub fn find_embedded_artwork_image(&self) -> Option<(ApicType, ImageFormat, &[u8])> {
        let Self(mp4_tag) = self;
        self::find_embedded_artwork_image(mp4_tag)
    }

    pub fn import_audio_content(&mut self, importer: &mut Importer) -> AudioContentMetadata {
        let Self(mp4_tag) = self;
        let duration = mp4_tag.duration().map(Into::into);
        let channels = mp4_tag.channel_config().map(read_channels);
        let sample_rate = mp4_tag
            .sample_rate()
            .as_ref()
            .map(Mp4SampleRate::hz)
            .map(|hz| SampleRateHz::from_inner(hz as SamplesPerSecond));
        let bitrate = mp4_tag.avg_bitrate().and_then(read_bitrate);
        let loudness = mp4_tag
            .strings_of(&IDENT_REPLAYGAIN_TRACK_GAIN)
            .next()
            .and_then(|input| importer.import_replay_gain(input));
        let encoder = mp4_tag
            .take_encoder()
            .and_then(trimmed_non_empty_from_owned)
            .map(Into::into);
        AudioContentMetadata {
            duration,
            channels,
            sample_rate,
            bitrate,
            loudness,
            encoder,
        }
    }

    pub fn import_into_track(
        mut self,
        importer: &mut Importer,
        config: &ImportTrackConfig,
        track: &mut Track,
    ) -> Result<()> {
        if track
            .media_source
            .content
            .metadata_flags
            .update(ContentMetadataFlags::UNRELIABLE)
        {
            let audio_content = self.import_audio_content(importer);
            track.media_source.content.metadata = ContentMetadata::Audio(audio_content);
        }

        let Self(mut mp4_tag) = self;

        track.media_source.advisory_rating =
            mp4_tag
                .advisory_rating()
                .map(|advisory_rating| match advisory_rating {
                    Mp4AdvisoryRating::Inoffensive => AdvisoryRating::Unrated,
                    Mp4AdvisoryRating::Clean => AdvisoryRating::Clean,
                    Mp4AdvisoryRating::Explicit => AdvisoryRating::Explicit,
                });

        let mut tempo_bpm_non_fractional = false;
        let tempo_bpm = mp4_tag
            .strings_of(&IDENT_BPM)
            .flat_map(|input| importer.import_tempo_bpm(input))
            .next()
            .or_else(|| {
                mp4_tag.bpm().and_then(|bpm| {
                    tempo_bpm_non_fractional = true;
                    let bpm = TempoBpm::from_inner(bpm.into());
                    bpm.is_valid().then(|| bpm)
                })
            });
        if let Some(tempo_bpm) = tempo_bpm {
            debug_assert!(tempo_bpm.is_valid());
            track.metrics.tempo_bpm = Some(tempo_bpm);
            track.metrics.flags.set(
                MetricsFlags::TEMPO_BPM_NON_FRACTIONAL,
                tempo_bpm_non_fractional,
            );
        } else {
            track.metrics.tempo_bpm = None;
        }

        let key_signature = mp4_tag
            .strings_of(&IDENT_INITIAL_KEY)
            // alternative name (conforms to Rapid Evolution)
            .chain(mp4_tag.strings_of(&KEY_IDENT))
            .flat_map(|input| importer.import_key_signature(input))
            .next();
        if let Some(key_signature) = key_signature {
            track.metrics.key_signature = key_signature;
        } else {
            track.metrics.key_signature = KeySignature::unknown();
        }

        // Track titles
        let mut track_titles = Vec::with_capacity(4);
        if let Some(title) = mp4_tag
            .take_title()
            .and_then(|name| ingest_title_from_owned(name, TitleKind::Main))
        {
            track_titles.push(title);
        }
        if let Some(title) = mp4_tag
            .take_strings_of(&IDENT_SUBTITLE)
            .next()
            .and_then(|name| ingest_title_from_owned(name, TitleKind::Sub))
        {
            track_titles.push(title);
        }
        if let Some(title) = mp4_tag
            .take_work()
            .and_then(|name| ingest_title_from_owned(name, TitleKind::Work))
        {
            track_titles.push(title);
        }
        if let Some(title) = mp4_tag
            .take_movement()
            .and_then(|name| ingest_title_from_owned(name, TitleKind::Movement))
        {
            track_titles.push(title);
        }
        let track_titles = importer.finish_import_of_titles(TrackScope::Track, track_titles);
        track.titles = track_titles;

        // Track actors
        let mut track_actors = Vec::with_capacity(8);
        for name in mp4_tag.take_artists() {
            push_next_actor_role_name(&mut track_actors, ActorRole::Artist, name);
        }
        for name in mp4_tag.take_composers() {
            push_next_actor_role_name(&mut track_actors, ActorRole::Composer, name);
        }
        for name in mp4_tag.take_strings_of(&IDENT_PRODUCER) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Producer, name);
        }
        for name in mp4_tag.take_strings_of(&IDENT_REMIXER) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Remixer, name);
        }
        for name in mp4_tag.take_strings_of(&IDENT_MIXER) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Mixer, name);
        }
        for name in mp4_tag.take_strings_of(&IDENT_ENGINEER) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Engineer, name);
        }
        for name in mp4_tag.take_lyricists() {
            push_next_actor_role_name(&mut track_actors, ActorRole::Lyricist, name);
        }
        for name in mp4_tag.take_strings_of(&IDENT_CONDUCTOR) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Conductor, name);
        }
        for name in mp4_tag.take_strings_of(&IDENT_DIRECTOR) {
            push_next_actor_role_name(&mut track_actors, ActorRole::Director, name);
        }
        let track_actors = importer.finish_import_of_actors(TrackScope::Track, track_actors);
        track.actors = track_actors;

        let mut album = track.album.untie_replace(Default::default());

        // Album titles
        let mut album_titles = Vec::with_capacity(1);
        if let Some(title) = mp4_tag
            .take_album()
            .and_then(|name| ingest_title_from_owned(name, TitleKind::Main))
        {
            album_titles.push(title);
        }
        let album_titles = importer.finish_import_of_titles(TrackScope::Album, album_titles);
        album.titles = album_titles;

        // Album actors
        let mut album_actors = Vec::with_capacity(4);
        for name in mp4_tag.take_album_artists() {
            push_next_actor_role_name(&mut album_actors, ActorRole::Artist, name);
        }
        let album_actors = importer.finish_import_of_actors(TrackScope::Album, album_actors);
        album.actors = album_actors;

        // Album properties
        if mp4_tag.compilation() {
            album.kind = AlbumKind::Compilation;
        } else {
            album.kind = AlbumKind::Unknown;
        }

        track.album = Canonical::tie(album);

        // Dedicated release dates are not available, only a generic recording date
        track.recorded_at = mp4_tag
            .take_strings_of(&IDENT_YEAR)
            .filter_map(|value| {
                importer.import_year_tag_from_field(&IDENT_YEAR.to_string(), &value)
            })
            .next();

        track.copyright = mp4_tag
            .take_copyright()
            .and_then(trimmed_non_empty_from_owned)
            .map(Into::into);
        track.publisher = mp4_tag
            .take_strings_of(&IDENT_LABEL)
            .filter_map(trimmed_non_empty_from_owned)
            .next()
            .map(Into::into);

        let mut tags_map = TagsMap::default();

        // Grouping tags
        importer.import_faceted_tags_from_label_values(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_GROUPING,
            mp4_tag.take_groupings().map(Into::into),
        );

        // Import gigtags from raw grouping tags before any other tags.
        #[cfg(feature = "gigtags")]
        if config.flags.contains(ImportTrackFlags::GIGTAGS) {
            if let Some(faceted_tags) = tags_map.take_faceted_tags(&FACET_ID_GROUPING) {
                tags_map = crate::util::gigtags::import_from_faceted_tags(faceted_tags);
            }
        }

        // Genre tags (custom + standard)
        {
            // Prefer custom genre tags
            let tag_mapping_config = config.faceted_tag_mapping.get(FACET_ID_GENRE.value());
            let mut next_score_value = TagScore::default_value();
            let mut plain_tags = Vec::with_capacity(8);
            if mp4_tag.custom_genres().next().is_some() {
                for genre in mp4_tag.take_custom_genres() {
                    importer.import_plain_tags_from_joined_label_value(
                        tag_mapping_config,
                        &mut next_score_value,
                        &mut plain_tags,
                        genre,
                    );
                }
            }
            if plain_tags.is_empty() {
                // Import legacy/standard genres only as a fallback
                for genre_id in mp4_tag.standard_genres() {
                    let genre_id = usize::from(genre_id);
                    if genre_id < STANDARD_GENRES.len() {
                        let genre = STANDARD_GENRES[genre_id];
                        importer.import_plain_tags_from_joined_label_value(
                            tag_mapping_config,
                            &mut next_score_value,
                            &mut plain_tags,
                            genre,
                        );
                    }
                }
            }
            tags_map.update_faceted_plain_tags_by_label_ordering(&FACET_ID_GENRE, plain_tags);
        }

        // Mood tags
        importer.import_faceted_tags_from_label_values(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_MOOD,
            mp4_tag.take_strings_of(&IDENT_MOOD).map(Into::into),
        );

        // Comment tag
        importer.import_faceted_tags_from_label_values(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_COMMENT,
            mp4_tag.take_strings_of(&IDENT_COMMENT).map(Into::into),
        );

        // Description tag
        importer.import_faceted_tags_from_label_values(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_DESCRIPTION,
            mp4_tag.take_strings_of(&IDENT_DESCRIPTION).map(Into::into),
        );

        // ISRC tag
        importer.import_faceted_tags_from_label_values(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_ISRC,
            mp4_tag.take_isrc().into_iter().map(Into::into),
        );

        // iTunes XID tags
        importer.import_faceted_tags_from_label_values(
            &mut tags_map,
            &config.faceted_tag_mapping,
            &FACET_ID_XID,
            mp4_tag.take_strings_of(&IDENT_XID).map(Into::into),
        );

        debug_assert!(track.tags.is_empty());
        track.tags = Canonical::tie(tags_map.into());

        // Indexes (in pairs)
        // Import both values consistently if any of them is available!
        if mp4_tag.track_number().is_some() || mp4_tag.total_tracks().is_some() {
            track.indexes.track.number = mp4_tag.track_number();
            track.indexes.track.total = mp4_tag.total_tracks();
        } else {
            // Reset
            track.indexes.track = Default::default();
        }
        if mp4_tag.disc_number().is_some() || mp4_tag.total_discs().is_some() {
            track.indexes.disc.number = mp4_tag.disc_number();
            track.indexes.disc.total = mp4_tag.total_discs();
        } else {
            // Reset
            track.indexes.disc = Default::default();
        }
        if mp4_tag.movement_index().is_some() || mp4_tag.movement_count().is_some() {
            track.indexes.movement.number = mp4_tag.movement_index();
            track.indexes.movement.total = mp4_tag.movement_count();
        } else {
            // Reset
            track.indexes.movement = Default::default();
        }

        // Artwork
        if config
            .flags
            .contains(ImportTrackFlags::METADATA_EMBEDDED_ARTWORK)
        {
            let artwork = if let Some((apic_type, image_format, image_data)) =
                find_embedded_artwork_image(&mp4_tag)
            {
                let (artwork, _, issues) = try_ingest_embedded_artwork_image(
                    apic_type,
                    image_data,
                    Some(image_format),
                    None,
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

        #[cfg(feature = "serato-markers")]
        #[allow(clippy::blocks_in_if_conditions)]
        if config.flags.contains(ImportTrackFlags::SERATO_MARKERS) {
            let mut serato_tags = triseratops::tag::TagContainer::new();
            let mut parsed = false;

            if let Some(data) = mp4_tag.data_of(&SERATO_MARKERS_IDENT).next() {
                match data {
                    Data::Utf8(input) => {
                        if serato_tags
                            .parse_markers(input.as_bytes(), triseratops::tag::TagFormat::MP4)
                            .map_err(|err| {
                                importer
                                    .add_issue(format!("Failed to parse Serato Markers: {err}"));
                            })
                            .is_ok()
                        {
                            parsed = true;
                        }
                    }
                    data => {
                        importer.add_issue(format!("Unexpected data for Serato Markers: {data:?}"));
                    }
                }
            }

            if let Some(data) = mp4_tag.data_of(&SERATO_MARKERS2_IDENT).next() {
                match data {
                    Data::Utf8(input) => {
                        if serato_tags
                            .parse_markers2(input.as_bytes(), triseratops::tag::TagFormat::MP4)
                            .map_err(|err| {
                                importer
                                    .add_issue(format!("Failed to parse Serato Markers2: {err}"));
                            })
                            .is_ok()
                        {
                            parsed = true;
                        }
                    }
                    data => {
                        importer
                            .add_issue(format!("Unexpected data for Serato Markers2: {data:?}"));
                    }
                }
            }

            if parsed {
                track.cues = Canonical::tie(crate::util::serato::import_cues(&serato_tags));
                track.color = crate::util::serato::import_track_color(&serato_tags);
            }
        }

        Ok(())
    }
}

fn export_filtered_actor_names(
    mp4_tag: &mut Mp4Tag,
    ident: impl Ident + Into<DataIdent>,
    actor_names: FilteredActorNames<'_>,
) {
    match actor_names {
        FilteredActorNames::Summary(name) => {
            mp4_tag.set_all_data(ident, once(Data::Utf8(name.to_owned())));
        }
        FilteredActorNames::Primary(names) => {
            mp4_tag.set_all_data(
                ident,
                names.into_iter().map(|name| Data::Utf8(name.to_owned())),
            );
        }
    }
}

fn export_faceted_tags(
    mp4_tag: &mut Mp4Tag,
    ident: impl Ident + Into<DataIdent>,
    config: Option<&TagMappingConfig>,
    tags: Vec<PlainTag>,
) {
    if let Some(config) = config {
        let joined_labels = config
            .join_labels(
                tags.iter()
                    .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(AsRef::as_ref)),
            )
            .to_owned();
        mp4_tag.set_all_data(ident, joined_labels.map(|s| Data::Utf8(s.into())));
    } else {
        mp4_tag.set_all_data(
            ident,
            tags.into_iter().filter_map(|PlainTag { label, score: _ }| {
                label.map(|label| Data::Utf8(label.into()))
            }),
        );
    }
}

pub fn export_track_to_path(
    path: &Path,
    config: &ExportTrackConfig,
    track: &mut Track,
) -> Result<bool> {
    let mp4_tag_orig = Mp4Tag::read_from_path(path).map_err(map_mp4ameta_err)?;

    let mut mp4_tag = mp4_tag_orig.clone();

    // Audio properties
    match &track.media_source.content.metadata {
        ContentMetadata::Audio(audio) => {
            if let Some(formatted_track_gain) = audio.loudness.and_then(format_valid_replay_gain) {
                mp4_tag.set_all_data(
                    IDENT_REPLAYGAIN_TRACK_GAIN,
                    once(Data::Utf8(formatted_track_gain)),
                );
            } else {
                mp4_tag.remove_data_of(&IDENT_REPLAYGAIN_TRACK_GAIN);
            }
            if let Some(encoder) = &audio.encoder {
                mp4_tag.set_encoder(encoder);
            }
        }
    }

    // Music: Tempo/BPM
    if let Some(formatted_bpm) = format_validated_tempo_bpm(&mut track.metrics.tempo_bpm) {
        mp4_tag.set_all_data(IDENT_BPM, once(Data::Utf8(formatted_bpm)));
        mp4_tag.set_bpm(
            track
                .metrics
                .tempo_bpm
                .expect("valid bpm")
                .to_inner()
                .round()
                .max(TempoBpm::from_inner(u16::MAX.into()).to_inner()) as u16,
        );
    } else {
        mp4_tag.remove_bpm();
        mp4_tag.remove_data_of(&IDENT_BPM);
    }

    // Music: Key
    if track.metrics.key_signature.is_unknown() {
        mp4_tag.remove_data_of(&IDENT_INITIAL_KEY);
        mp4_tag.remove_data_of(&KEY_IDENT);
    } else {
        // TODO: Write a custom key code string according to config
        mp4_tag.set_all_data(
            IDENT_INITIAL_KEY,
            once(Data::Utf8(track.metrics.key_signature.to_string())),
        );
        if mp4_tag.data_of(&KEY_IDENT).next().is_some() {
            // Write non-standard key atom only if already present
            mp4_tag.set_all_data(
                KEY_IDENT,
                once(Data::Utf8(track.metrics.key_signature.to_string())),
            );
        }
    }

    // Track titles
    if let Some(title) = Titles::main_title(track.titles.iter()) {
        mp4_tag.set_title(title.name.to_owned());
    } else {
        mp4_tag.remove_title();
    }
    let track_subtitles = Titles::filter_kind(track.titles.iter(), TitleKind::Sub).peekable();
    mp4_tag.set_all_data(
        IDENT_SUBTITLE,
        track_subtitles.map(|subtitle| Data::Utf8(subtitle.name.to_owned())),
    );
    let mut track_movements =
        Titles::filter_kind(track.titles.iter(), TitleKind::Movement).peekable();
    if track_movements.peek().is_some() {
        let movement = track_movements.next().unwrap();
        // Only a single movement is supported
        debug_assert!(track_movements.peek().is_none());
        mp4_tag.set_movement(movement.name.to_owned());
    } else {
        mp4_tag.remove_movement();
    }
    let mut track_works = Titles::filter_kind(track.titles.iter(), TitleKind::Work).peekable();
    if track_works.peek().is_some() {
        let work = track_works.next().unwrap();
        // Only a single work is supported
        debug_assert!(track_works.peek().is_none());
        mp4_tag.set_work(work.name.to_owned());
    } else {
        mp4_tag.remove_work();
    }

    // Track actors
    export_filtered_actor_names(
        &mut mp4_tag,
        IDENT_ARTIST,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Artist),
    );
    export_filtered_actor_names(
        &mut mp4_tag,
        IDENT_COMPOSER,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Composer),
    );
    export_filtered_actor_names(
        &mut mp4_tag,
        IDENT_DIRECTOR,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Director),
    );
    export_filtered_actor_names(
        &mut mp4_tag,
        IDENT_LYRICIST,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Lyricist),
    );
    export_filtered_actor_names(
        &mut mp4_tag,
        IDENT_CONDUCTOR,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Conductor),
    );
    export_filtered_actor_names(
        &mut mp4_tag,
        IDENT_ENGINEER,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Engineer),
    );
    export_filtered_actor_names(
        &mut mp4_tag,
        IDENT_MIXER,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Mixer),
    );
    export_filtered_actor_names(
        &mut mp4_tag,
        IDENT_PRODUCER,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Producer),
    );
    export_filtered_actor_names(
        &mut mp4_tag,
        IDENT_REMIXER,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Remixer),
    );

    // Album
    if let Some(title) = Titles::main_title(track.album.titles.iter()) {
        mp4_tag.set_album(title.name.to_owned());
    } else {
        mp4_tag.remove_album();
    }
    export_filtered_actor_names(
        &mut mp4_tag,
        IDENT_ALBUM_ARTIST,
        FilteredActorNames::new(track.album.actors.iter(), ActorRole::Artist),
    );
    match track.album.kind {
        AlbumKind::Unknown => {
            mp4_tag.remove_compilation();
        }
        AlbumKind::Compilation => {
            mp4_tag.set_compilation();
        }
        AlbumKind::Album | AlbumKind::Single => {
            // TODO: Set compilation flag to false!?
            mp4_tag.remove_compilation();
        }
    }

    // No distinction between recording and release date, i.e.
    // only the release date is stored.
    if let Some(recorded_at) = track.recorded_at {
        mp4_tag.set_year(recorded_at.to_string());
    } else {
        mp4_tag.remove_year();
    }
    if let Some(publisher) = &track.publisher {
        mp4_tag.set_all_data(IDENT_LABEL, once(Data::Utf8(publisher.to_owned())));
    } else {
        mp4_tag.remove_data_of(&IDENT_LABEL);
    }
    if let Some(copyright) = &track.copyright {
        mp4_tag.set_copyright(copyright);
    } else {
        mp4_tag.remove_copyright();
    }

    // Numbers
    if let Some(track_number) = track.indexes.track.number {
        mp4_tag.set_track_number(track_number);
    } else {
        mp4_tag.remove_track_number();
    }
    if let Some(track_total) = track.indexes.track.total {
        mp4_tag.set_total_tracks(track_total);
    } else {
        mp4_tag.remove_total_tracks();
    }
    if let Some(disc_number) = track.indexes.disc.number {
        mp4_tag.set_disc_number(disc_number);
    } else {
        mp4_tag.remove_disc_number();
    }
    if let Some(disc_total) = track.indexes.disc.total {
        mp4_tag.set_total_discs(disc_total);
    } else {
        mp4_tag.remove_total_discs();
    }
    if let Some(movement_number) = track.indexes.movement.number {
        mp4_tag.set_movement_index(movement_number);
    } else {
        mp4_tag.remove_movement_index();
    }
    if let Some(movement_total) = track.indexes.movement.total {
        mp4_tag.set_movement_count(movement_total);
    } else {
        mp4_tag.remove_movement_count();
    }

    // Export selected tags into dedicated fields
    let mut tags_map = TagsMap::from(track.tags.clone().untie());

    // Genre(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_GENRE) {
        // Overwrite standard genres with custom genres
        mp4_tag.remove_standard_genres();
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_GENRE,
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        // Preserve standard genres until overwritten by custom genres
        mp4_tag.remove_data_of(&IDENT_GENRE);
    }

    // Comment(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_COMMENT) {
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_COMMENT,
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        mp4_tag.remove_data_of(&IDENT_COMMENT);
    }

    // Description(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_DESCRIPTION)
    {
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_DESCRIPTION,
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        mp4_tag.remove_data_of(&IDENT_DESCRIPTION);
    }

    // Mood(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_MOOD) {
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_MOOD,
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        mp4_tag.remove_data_of(&IDENT_MOOD);
    }

    // ISRC(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_ISRC) {
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_ISRC,
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        mp4_tag.remove_data_of(&IDENT_ISRC);
    }

    // XID(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(&FACET_ID_XID) {
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_XID,
            config.faceted_tag_mapping.get(facet_id.value()),
            tags,
        );
    } else {
        mp4_tag.remove_data_of(&IDENT_XID);
    }

    // Grouping(s)
    {
        let facet_id = &FACET_ID_GROUPING;
        let mut tags = tags_map
            .take_faceted_tags(facet_id)
            .map(|FacetedTags { facet_id: _, tags }| tags)
            .unwrap_or_default();
        #[cfg(feature = "gigtags")]
        if config.flags.contains(ExportTrackFlags::GIGTAGS) {
            if let Err(err) = crate::util::gigtags::export_and_encode_remaining_tags_into(
                tags_map.into(),
                &mut tags,
            ) {
                log::error!("Failed to export gigitags: {err}");
            }
        }
        if tags.is_empty() {
            mp4_tag.remove_data_of(&IDENT_GROUPING);
        } else {
            export_faceted_tags(
                &mut mp4_tag,
                IDENT_GROUPING,
                config.faceted_tag_mapping.get(facet_id.value()),
                tags,
            );
        }
    }

    if mp4_tag == mp4_tag_orig {
        // Unmodified
        return Ok(false);
    }
    mp4_tag.write_to_path(path).map_err(map_mp4ameta_err)?;
    // Modified
    Ok(true)
}
