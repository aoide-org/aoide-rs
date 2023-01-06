// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, ops::Not as _};

use lofty::{
    Accessor, AudioFile, FileProperties, ItemKey, PictureType, Tag, TaggedFile, TaggedFileExt as _,
};
use semval::IsValid;

use aoide_core::{
    audio::{
        channel::ChannelCount,
        signal::{BitrateBps, SampleRateHz},
        DurationMs,
    },
    media::{
        artwork::{ApicType, Artwork},
        content::{AudioContentMetadata, ContentMetadata, ContentMetadataFlags},
    },
    tag::{Score as TagScore, TagsMap},
    track::{
        actor::Role as ActorRole,
        album::Kind as AlbumKind,
        metric::MetricsFlags,
        tag::{
            FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING,
            FACET_ID_ISRC, FACET_ID_MOOD,
        },
        title::Kind as TitleKind,
        Track,
    },
    util::{canonical::Canonical, string::trimmed_non_empty_from},
};

use crate::{
    io::import::{ImportTrackConfig, ImportTrackFlags, Importer, TrackScope},
    util::{
        digest::MediaDigest, ingest_title_from, push_next_actor_role_name_from,
        try_ingest_embedded_artwork_image,
    },
    Result,
};

pub(crate) mod aiff;

pub(crate) mod flac;

pub(crate) mod id3v2;

pub(crate) mod mp4;

pub(crate) mod mpeg;

pub(crate) mod ogg;

pub(crate) mod opus;

pub(crate) mod vorbis;

const ENCODER_FIELD_SEPARATOR: &str = "|";

fn import_audio_content_from_file_properties(properties: &FileProperties) -> AudioContentMetadata {
    let bitrate = properties
        .audio_bitrate()
        .map(|kbps| BitrateBps::from_inner(f64::from(kbps) * 1000.0))
        .filter(IsValid::is_valid);
    let channels = properties
        .channels()
        .map(|num_channels| ChannelCount(num_channels.into()).into())
        .filter(IsValid::is_valid);
    let duration_ms = properties.duration().as_secs_f64() * 1000.0;
    let duration = Some(DurationMs::from_inner(duration_ms)).filter(IsValid::is_valid);
    let sample_rate = properties
        .sample_rate()
        .map(|hz| SampleRateHz::from_inner(hz.into()))
        .filter(IsValid::is_valid);
    AudioContentMetadata {
        bitrate,
        channels,
        duration,
        sample_rate,
        encoder: None,
        loudness: None,
    }
}

pub(crate) fn take_primary_or_first_tag(tagged_file: &mut TaggedFile) -> Option<Tag> {
    if let Some(tag) = tagged_file.remove(tagged_file.primary_tag_type()) {
        return Some(tag);
    }
    let Some(first_tag_type) = tagged_file.first_tag().map(|tag| tag.tag_type()) else {
        return None;
    };
    tagged_file.remove(first_tag_type)
}

fn apic_type_from_picture_type(picture_type: PictureType) -> Option<ApicType> {
    let apic_type = match picture_type {
        PictureType::Artist => ApicType::Artist,
        PictureType::Band => ApicType::Band,
        PictureType::BandLogo => ApicType::BandLogo,
        PictureType::BrightFish => ApicType::BrightFish,
        PictureType::Composer => ApicType::Composer,
        PictureType::Conductor => ApicType::Conductor,
        PictureType::CoverBack => ApicType::CoverBack,
        PictureType::CoverFront => ApicType::CoverFront,
        PictureType::DuringPerformance => ApicType::DuringPerformance,
        PictureType::DuringRecording => ApicType::DuringRecording,
        PictureType::Icon => ApicType::Icon,
        PictureType::Illustration => ApicType::Illustration,
        PictureType::LeadArtist => ApicType::LeadArtist,
        PictureType::Leaflet => ApicType::Leaflet,
        PictureType::Lyricist => ApicType::Lyricist,
        PictureType::Media => ApicType::Media,
        PictureType::Other => ApicType::Other,
        PictureType::OtherIcon => ApicType::OtherIcon,
        PictureType::PublisherLogo => ApicType::PublisherLogo,
        PictureType::RecordingLocation => ApicType::RecordingLocation,
        PictureType::ScreenCapture => ApicType::ScreenCapture,
        PictureType::Undefined(_) => {
            return None;
        }
        _ => {
            // non-exhaustive enum
            log::error!("Unhandled picture type: {picture_type:?}");
            return None;
        }
    };
    Some(apic_type)
}

#[must_use]
pub(crate) fn find_embedded_artwork_image(tag: &Tag) -> Option<(ApicType, &str, &[u8])> {
    tag.pictures()
        .iter()
        .filter_map(|p| {
            if p.pic_type() == PictureType::CoverFront {
                Some((ApicType::CoverFront, p))
            } else {
                None
            }
        })
        .chain(tag.pictures().iter().filter_map(|p| {
            if p.pic_type() == PictureType::Media {
                Some((ApicType::Media, p))
            } else {
                None
            }
        }))
        .chain(tag.pictures().iter().filter_map(|p| {
            if p.pic_type() == PictureType::Leaflet {
                Some((ApicType::Leaflet, p))
            } else {
                None
            }
        }))
        .chain(tag.pictures().iter().filter_map(|p| {
            if p.pic_type() == PictureType::Other {
                Some((ApicType::Other, p))
            } else {
                None
            }
        }))
        // otherwise take the first picture that could be parsed
        .chain(tag.pictures().iter().map(|p| {
            (
                apic_type_from_picture_type(p.pic_type()).unwrap_or(ApicType::Other),
                p,
            )
        }))
        .map(|(apic_type, p)| (apic_type, p.mime_type().as_str(), p.data()))
        .next()
}

pub(crate) fn import_embedded_artwork(
    importer: &mut Importer,
    tag: &Tag,
    mut media_digest: MediaDigest,
) -> Result<Artwork> {
    let artwork = if let Some((apic_type, mime_type, image_data)) = find_embedded_artwork_image(tag)
    {
        let (artwork, _, issues) = try_ingest_embedded_artwork_image(
            apic_type,
            image_data,
            None,
            Some(mime_type),
            &mut media_digest,
        );
        issues
            .into_iter()
            .for_each(|message| importer.add_issue(message));
        artwork
    } else {
        Artwork::Missing
    };
    Ok(artwork)
}

pub(crate) fn import_tagged_file_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    mut tagged_file: TaggedFile,
    track: &mut Track,
) -> Result<()> {
    debug_assert!(config.flags.contains(ImportTrackFlags::METADATA));

    let tag = take_primary_or_first_tag(&mut tagged_file);
    if let Some(tag) = tag {
        log::debug!(
            "Importing track metadata from {tag_type:?} tag in {file_type:?} file",
            tag_type = tag.tag_type(),
            file_type = tagged_file.file_type(),
        );
        let file_properties = tagged_file.properties();
        import_file_tag_into_track(importer, config, file_properties, tag, track)?;
    }

    Ok(())
}

pub(crate) fn import_file_tag_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    file_properties: &FileProperties,
    mut tag: Tag,
    track: &mut Track,
) -> Result<()> {
    debug_assert_eq!(
        track.media_source.content.metadata,
        ContentMetadata::Audio(Default::default())
    );
    let audio_content = track
        .media_source
        .content
        .metadata_flags
        .update(ContentMetadataFlags::UNRELIABLE)
        .then(|| import_audio_content_from_file_properties(file_properties));
    if let Some(mut audio_content) = audio_content {
        // Import the remaining audio content properties
        debug_assert!(audio_content.encoder.is_none());
        let encoder_info: Vec<_> = [
            tag.get_string(&ItemKey::EncodedBy),
            tag.get_string(&ItemKey::EncoderSoftware),
            tag.get_string(&ItemKey::EncoderSettings),
        ]
        .into_iter()
        .flatten()
        .collect();
        audio_content.encoder = encoder_info
            .is_empty()
            .not()
            .then(|| encoder_info.join(ENCODER_FIELD_SEPARATOR));
        debug_assert!(audio_content.loudness.is_none());
        audio_content.loudness = tag
            .get_string(&ItemKey::ReplayGainTrackGain)
            .and_then(|input| importer.import_loudness_from_replay_gain(input));
        track.media_source.content.metadata = ContentMetadata::Audio(audio_content);
    }

    // Musical metrics
    let imported_tempo_bpm = tag
        .take_strings(&ItemKey::BPM)
        .flat_map(|input| importer.import_tempo_bpm(&input))
        .next();
    if let Some(imported_tempo_bpm) = &imported_tempo_bpm {
        // Assume that the tag is only capable of storing BPM values with
        // integer precision if the number has no fractional decimal digits.
        track.metrics.flags.set(
            MetricsFlags::TEMPO_BPM_NON_FRACTIONAL,
            imported_tempo_bpm.is_non_fractional(),
        );
    }
    track.metrics.tempo_bpm = imported_tempo_bpm.map(Into::into);
    track.metrics.key_signature = tag
        .take_strings(&ItemKey::InitialKey)
        .flat_map(|input| importer.import_key_signature(&input))
        .next();

    // Track titles
    let mut track_titles = Vec::with_capacity(4);
    if let Some(title) = tag
        .take_strings(&ItemKey::TrackTitle)
        .filter_map(|name| ingest_title_from(name, TitleKind::Main))
        .next()
    {
        track_titles.push(title);
    }
    if let Some(title) = tag
        .take_strings(&ItemKey::TrackTitle)
        .filter_map(|name| ingest_title_from(name, TitleKind::Sub))
        .next()
    {
        track_titles.push(title);
    }
    if let Some(title) = tag
        .take_strings(&ItemKey::Movement)
        .filter_map(|name| ingest_title_from(name, TitleKind::Movement))
        .next()
    {
        track_titles.push(title);
    }
    // TODO: Work?
    track.titles = importer.finish_import_of_titles(TrackScope::Track, track_titles);

    // Track actors
    let mut track_actors = Vec::with_capacity(8);
    for name in tag.take_strings(&ItemKey::TrackArtist) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Artist, name);
    }
    for name in tag.take_strings(&ItemKey::Arranger) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Arranger, name);
    }
    for name in tag.take_strings(&ItemKey::Composer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Composer, name);
    }
    for name in tag.take_strings(&ItemKey::Conductor) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Conductor, name);
    }
    // FIXME: <https://github.com/Serial-ATA/lofty-rs/pull/100>
    // for name in tag.take_strings(&ItemKey::Directory) {
    //     push_next_actor_role_name_from(&mut track_actors, ActorRole::Directory, name);
    // }
    for name in tag.take_strings(&ItemKey::Engineer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Engineer, name);
    }
    for name in tag.take_strings(&ItemKey::Lyricist) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Lyricist, name);
    }
    for name in tag.take_strings(&ItemKey::MixDj) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::MixDj, name);
    }
    for name in tag.take_strings(&ItemKey::MixEngineer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::MixEngineer, name);
    }
    for name in tag.take_strings(&ItemKey::Performer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Performer, name);
    }
    for name in tag.take_strings(&ItemKey::Producer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Producer, name);
    }
    for name in tag.take_strings(&ItemKey::Writer) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Writer, name);
    }

    let mut album = track.album.untie_replace(Default::default());

    // Album titles
    let mut album_titles = Vec::with_capacity(1);
    if let Some(title) = tag
        .take_strings(&ItemKey::AlbumTitle)
        .filter_map(|name| ingest_title_from(name, TitleKind::Main))
        .next()
    {
        album_titles.push(title);
    }
    album.titles = importer.finish_import_of_titles(TrackScope::Album, album_titles);

    // Album actors
    let mut album_actors = Vec::with_capacity(4);
    for name in tag.take_strings(&ItemKey::AlbumArtist) {
        push_next_actor_role_name_from(&mut album_actors, ActorRole::Artist, name);
    }
    album.actors = importer.finish_import_of_actors(TrackScope::Album, album_actors);

    if let Some(item) = tag.take(&ItemKey::FlagCompilation).next() {
        if let Some(kind) = item
            .value()
            .text()
            .and_then(|input| input.parse::<u8>().ok())
            .and_then(|value| match value {
                0 => Some(AlbumKind::NoCompilation),
                1 => Some(AlbumKind::Compilation),
                _ => None,
            })
        {
            debug_assert!(album.kind.is_none());
            album.kind = Some(kind);
        } else {
            importer.add_issue(format!("Unexpected compilation flag item: {item:?}"));
        }
    }

    track.album = Canonical::tie(album);

    // Indexes (in pairs)
    // Import both values consistently if any of them is available!
    // TODO: Verify u32 -> u16 conversions
    if tag.track().is_some() || tag.track_total().is_some() {
        track.indexes.track.number = tag.track().map(|val| val as _);
        track.indexes.track.total = tag.track_total().map(|val| val as _);
    } else {
        debug_assert_eq!(track.indexes.track, Default::default());
    }
    if tag.disk().is_some() || tag.disk_total().is_some() {
        track.indexes.disc.number = tag.disk().map(|val| val as _);
        track.indexes.disc.total = tag.disk_total().map(|val| val as _);
    } else {
        debug_assert_eq!(track.indexes.track, Default::default());
    }
    // TODO: Movement and work?

    track.copyright = tag
        .take_strings(&ItemKey::CopyrightMessage)
        .filter_map(trimmed_non_empty_from)
        .next()
        .map(Cow::into_owned);
    track.publisher = tag
        .take_strings(&ItemKey::Label)
        .filter_map(trimmed_non_empty_from)
        .next()
        .map(Cow::into_owned);
    if track.publisher.is_none() {
        tag.take_strings(&ItemKey::Publisher)
            .filter_map(trimmed_non_empty_from)
            .next()
            .map(Cow::into_owned);
    }

    if let Some(item) = tag.take(&ItemKey::ParentalAdvisory).next() {
        log::warn!("TODO: Handle item with parental advisory: {item:?}");
    }

    track.recorded_at = tag
        .take_strings(&ItemKey::RecordingDate)
        .filter_map(|input| importer.import_year_tag_from_field("RecordingDate", &input))
        .next();
    if track.recorded_at.is_none() {
        track.recorded_at = tag
            .take_strings(&ItemKey::Year)
            .filter_map(|input| importer.import_year_tag_from_field("Year", &input))
            .next();
    }
    track.released_at = tag
        .take_strings(&ItemKey::PodcastReleaseDate)
        .filter_map(|input| importer.import_year_tag_from_field("PodcastReleaseDate", &input))
        .next();
    track.released_orig_at = tag
        .take_strings(&ItemKey::OriginalReleaseDate)
        .filter_map(|input| importer.import_year_tag_from_field("OriginalReleaseDate", &input))
        .next();

    let mut tags_map = TagsMap::default();

    // Grouping tags
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_GROUPING,
        tag.take_strings(&ItemKey::ContentGroup).map(Into::into),
    );

    // Import gig tags from raw grouping tags before any other tags.
    #[cfg(feature = "gigtag")]
    if config.flags.contains(ImportTrackFlags::GIGTAGS) {
        if let Some(faceted_tags) = tags_map.take_faceted_tags(&FACET_ID_GROUPING) {
            tags_map = crate::util::gigtag::import_from_faceted_tags(faceted_tags);
        }
    }

    // Genre tags
    {
        let tag_mapping_config = config.faceted_tag_mapping.get(FACET_ID_GENRE.value());
        let mut next_score_value = TagScore::default_value();
        let mut plain_tags = Vec::with_capacity(8);
        for genre in tag.take_strings(&ItemKey::Genre) {
            importer.import_plain_tags_from_joined_label_value(
                tag_mapping_config,
                &mut next_score_value,
                &mut plain_tags,
                genre,
            );
        }
        tags_map.update_faceted_plain_tags_by_label_ordering(&FACET_ID_GENRE, plain_tags);
    }

    // Mood tags
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_MOOD,
        tag.take_strings(&ItemKey::Mood).map(Into::into),
    );

    // Comment tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_COMMENT,
        tag.take_strings(&ItemKey::Comment).map(Into::into),
    );

    // Description tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_DESCRIPTION,
        tag.take_strings(&ItemKey::Description).map(Into::into),
    );

    // ISRC tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        &FACET_ID_ISRC,
        tag.take_strings(&ItemKey::ISRC).map(Into::into),
    );

    // XID tag
    // FIXME: <https://github.com/Serial-ATA/lofty-rs/pull/98>
    // importer.import_faceted_tags_from_label_values(
    //     &mut tags_map,
    //     &config.faceted_tag_mapping,
    //     &FACET_ID_XID,
    //     tag.take_strings(&ItemKey::AppleItunesXID).map(Into::into),
    // );

    debug_assert!(track.tags.is_empty());
    track.tags = Canonical::tie(tags_map.into());

    // Artwork
    if config
        .flags
        .contains(ImportTrackFlags::METADATA_EMBEDDED_ARTWORK)
    {
        let artwork = import_embedded_artwork(importer, &tag, config.flags.new_artwork_digest())?;
        track.media_source.artwork = Some(artwork);
    }

    Ok(())
}
