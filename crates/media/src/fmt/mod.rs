// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, ops::Not as _};

use lofty::{
    Accessor, AudioFile, FileProperties, ItemKey, ItemValue, MimeType, ParseOptions, Picture,
    PictureType, Tag, TagItem, TagType, TaggedFile, TaggedFileExt as _,
};
use semval::IsValid;

use aoide_core::{
    audio::{
        channel::ChannelCount,
        signal::{BitrateBps, SampleRateHz},
        DurationMs,
    },
    media::{
        artwork::{ApicType, Artwork, ArtworkImage, EmbeddedArtwork},
        content::{AudioContentMetadata, ContentMetadata, ContentMetadataFlags},
    },
    tag::{FacetKey, FacetedTags, Label, PlainTag, Score as TagScore, TagsMap},
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
    util::{canonical::Canonical, string::trimmed_non_empty_from},
};

use crate::{
    io::{
        export::{ExportTrackConfig, ExportTrackFlags, FilteredActorNames},
        import::{ImportTrackConfig, ImportTrackFlags, Importer, TrackScope},
    },
    util::{
        artwork::{
            try_ingest_embedded_artwork_image, ReplaceEmbeddedArtworkImage,
            ReplaceOtherEmbeddedArtworkImages,
        },
        digest::MediaDigest,
        format_valid_replay_gain, format_validated_tempo_bpm, ingest_title_from,
        key_signature_as_str, push_next_actor_role_name_from,
        tag::TagMappingConfig,
        TempoBpmFormat,
    },
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

pub(crate) fn parse_options() -> ParseOptions {
    ParseOptions::new()
        .parsing_mode(lofty::ParsingMode::Relaxed)
        .read_properties(true)
}

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
    let Some(first_tag_type) = tagged_file.first_tag().map(Tag::tag_type) else {
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

fn picture_type_from_apic_type(apic_type: ApicType) -> PictureType {
    match apic_type {
        ApicType::Artist => PictureType::Artist,
        ApicType::Band => PictureType::Band,
        ApicType::BandLogo => PictureType::BandLogo,
        ApicType::BrightFish => PictureType::BrightFish,
        ApicType::Composer => PictureType::Composer,
        ApicType::Conductor => PictureType::Conductor,
        ApicType::CoverBack => PictureType::CoverBack,
        ApicType::CoverFront => PictureType::CoverFront,
        ApicType::DuringPerformance => PictureType::DuringPerformance,
        ApicType::DuringRecording => PictureType::DuringRecording,
        ApicType::Icon => PictureType::Icon,
        ApicType::Illustration => PictureType::Illustration,
        ApicType::LeadArtist => PictureType::LeadArtist,
        ApicType::Leaflet => PictureType::Leaflet,
        ApicType::Lyricist => PictureType::Lyricist,
        ApicType::Media => PictureType::Media,
        ApicType::Other => PictureType::Other,
        ApicType::OtherIcon => PictureType::OtherIcon,
        ApicType::PublisherLogo => PictureType::PublisherLogo,
        ApicType::RecordingLocation => PictureType::RecordingLocation,
        ApicType::ScreenCapture => PictureType::ScreenCapture,
    }
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
) -> Artwork {
    let artwork = if let Some((apic_type, mime_type, image_data)) = find_embedded_artwork_image(tag)
    {
        let (artwork, _, issues) = try_ingest_embedded_artwork_image(
            apic_type,
            image_data,
            None,
            Some(mime_type),
            &mut media_digest,
        );
        for issue in issues {
            importer.add_issue(issue);
        }
        artwork
    } else {
        Artwork::Missing
    };
    artwork
}

pub(crate) fn import_tagged_file_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    mut tagged_file: TaggedFile,
    track: &mut Track,
) {
    let tag = take_primary_or_first_tag(&mut tagged_file);
    if let Some(tag) = tag {
        log::debug!(
            "Importing track metadata from {tag_type:?} tag in {file_type:?} file",
            tag_type = tag.tag_type(),
            file_type = tagged_file.file_type(),
        );
        let file_properties = tagged_file.properties();
        import_file_tag_into_track(importer, config, file_properties, tag, track);
    }
}

// Compatibility hacks for mapping ItemKey::ContentGroup and ItemKey::Work
#[derive(Debug)]
struct Compatibility {
    primary_content_group_item_key: ItemKey,
    secondary_content_group_item_key: Option<ItemKey>,
    primary_work_item_key: ItemKey,
    secondary_work_item_key: Option<ItemKey>,
}

impl Compatibility {
    fn import(tage_type: TagType, flags: ImportTrackFlags) -> Self {
        Self::new(
            tage_type,
            flags.contains(ImportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1),
        )
    }

    fn export(tage_type: TagType, flags: ExportTrackFlags) -> Self {
        Self::new(
            tage_type,
            flags.contains(ExportTrackFlags::COMPATIBILITY_ID3V2_APPLE_GRP1),
        )
    }

    fn new(tage_type: TagType, apple_grp1: bool) -> Self {
        let primary_content_group_item_key;
        let secondary_content_group_item_key;
        let primary_work_item_key;
        let secondary_work_item_key;
        if matches!(tage_type, TagType::ID3v2) {
            primary_content_group_item_key = ItemKey::AppleId3v2ContentGroup; // GRP1
            primary_work_item_key = ItemKey::Work; // TXXX:WORK
            if apple_grp1 {
                secondary_content_group_item_key = None;
                secondary_work_item_key = Some(ItemKey::ContentGroup); // TIT1
            } else {
                secondary_content_group_item_key = Some(ItemKey::ContentGroup); // TIT1
                secondary_work_item_key = None;
            }
        } else {
            primary_content_group_item_key = ItemKey::ContentGroup;
            secondary_content_group_item_key = None;
            primary_work_item_key = ItemKey::Work;
            secondary_work_item_key = None;
        }
        Self {
            primary_content_group_item_key,
            secondary_content_group_item_key,
            primary_work_item_key,
            secondary_work_item_key,
        }
    }
}

#[allow(clippy::too_many_lines)] // TODO
pub(crate) fn import_file_tag_into_track(
    importer: &mut Importer,
    config: &ImportTrackConfig,
    file_properties: &FileProperties,
    mut tag: Tag,
    track: &mut Track,
) {
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

    if !config.flags.contains(ImportTrackFlags::METADATA) {
        return;
    }

    let compatibility = Compatibility::import(tag.tag_type(), config.flags);

    // Musical metrics
    track.metrics.tempo_bpm = None;
    for imported_tempo_bpm in tag
        .take_strings(&ItemKey::BPM)
        .filter_map(|input| importer.import_tempo_bpm(&input))
    {
        let is_non_fractional = imported_tempo_bpm.is_non_fractional();
        track.metrics.tempo_bpm = Some(imported_tempo_bpm.into());
        if is_non_fractional {
            // Assume that the tag is only capable of storing BPM values with
            // integer precision if the number has no fractional decimal digits.
            track
                .metrics
                .flags
                .set(MetricsFlags::TEMPO_BPM_NON_FRACTIONAL, true);
            // Continue and try to find a more precise, fractional bpm
        } else {
            track
                .metrics
                .flags
                .set(MetricsFlags::TEMPO_BPM_NON_FRACTIONAL, false);
            // Abort after finding the first fractional bpm
            break;
        }
    }
    track.metrics.key_signature = tag
        .take_strings(&ItemKey::InitialKey)
        .find_map(|input| importer.import_key_signature(&input));

    // Track titles
    let mut track_titles = Vec::with_capacity(4);
    if let Some(title) = tag
        .take_strings(&ItemKey::TrackTitle)
        .find_map(|name| ingest_title_from(name, TitleKind::Main))
    {
        track_titles.push(title);
    }
    if let Some(title) = tag
        .take_strings(&ItemKey::TrackTitle)
        .find_map(|name| ingest_title_from(name, TitleKind::Sub))
    {
        track_titles.push(title);
    }
    if let Some(title) = tag
        .take_strings(&ItemKey::Movement)
        .find_map(|name| ingest_title_from(name, TitleKind::Movement))
    {
        track_titles.push(title);
    }
    let primary_work_title = tag
        .take_strings(&compatibility.primary_work_item_key)
        .find_map(|name| ingest_title_from(name, TitleKind::Work));
    if let Some(work_title) = primary_work_title.or_else(|| {
        compatibility
            .secondary_work_item_key
            .and_then(|secondary_work_item_key| {
                tag.take_strings(&secondary_work_item_key)
                    .find_map(|name| ingest_title_from(name, TitleKind::Work))
            })
    }) {
        track_titles.push(work_title);
    }
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
    for name in tag.take_strings(&ItemKey::Director) {
        push_next_actor_role_name_from(&mut track_actors, ActorRole::Director, name);
    }
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
    track.actors = Canonical::tie(track_actors);

    let mut album = track.album.untie_replace(Default::default());

    // Album titles
    let mut album_titles = Vec::with_capacity(1);
    if let Some(title) = tag
        .take_strings(&ItemKey::AlbumTitle)
        .find_map(|name| ingest_title_from(name, TitleKind::Main))
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

    track.copyright = tag
        .take_strings(&ItemKey::CopyrightMessage)
        .find_map(trimmed_non_empty_from)
        .map(Cow::into_owned);
    track.publisher = tag
        .take_strings(&ItemKey::Label)
        .find_map(trimmed_non_empty_from)
        .map(Cow::into_owned);
    if track.publisher.is_none() {
        tag.take_strings(&ItemKey::Publisher)
            .find_map(trimmed_non_empty_from)
            .map(Cow::into_owned);
    }

    // Index pairs
    // Import both values consistently if any of them is available!
    // TODO: Verify u32 -> u16 conversions
    let track_number = tag
        .track()
        .map(TryFrom::try_from)
        .transpose()
        .ok()
        .flatten();
    let track_total = tag
        .track_total()
        .map(TryFrom::try_from)
        .transpose()
        .ok()
        .flatten();
    if track_number.is_some() || track_total.is_some() {
        track.indexes.track.number = track_number;
        track.indexes.track.total = track_total;
    } else {
        debug_assert_eq!(track.indexes.track, Default::default());
    }
    let disc_number = tag.disk().map(TryFrom::try_from).transpose().ok().flatten();
    let disc_total = tag
        .disk_total()
        .map(TryFrom::try_from)
        .transpose()
        .ok()
        .flatten();
    if disc_number.is_some() || disc_total.is_some() {
        track.indexes.disc.number = disc_number;
        track.indexes.disc.total = disc_total;
    } else {
        debug_assert_eq!(track.indexes.disc, Default::default());
    }
    let movement_number =
        tag.get_items(&ItemKey::MovementNumber)
            .find_map(|item| match item.value() {
                ItemValue::Text(number) => number.parse::<u16>().ok(),
                _ => None,
            });
    let movement_total =
        tag.get_items(&ItemKey::MovementNumber)
            .find_map(|item| match item.value() {
                ItemValue::Text(number) => number.parse::<u16>().ok(),
                _ => None,
            });
    if movement_number.is_some() || movement_total.is_some() {
        track.indexes.movement.number = movement_number;
        track.indexes.movement.total = movement_total;
    } else {
        debug_assert_eq!(track.indexes.movement, Default::default());
    }

    track.recorded_at = tag
        .take_strings(&ItemKey::RecordingDate)
        .find_map(|input| importer.import_year_tag_from_field("RecordingDate", &input));
    if track.recorded_at.is_none() {
        track.recorded_at = tag
            .take_strings(&ItemKey::Year)
            .find_map(|input| importer.import_year_tag_from_field("Year", &input));
    }
    track.released_at = tag
        .take_strings(&ItemKey::PodcastReleaseDate)
        .find_map(|input| importer.import_year_tag_from_field("PodcastReleaseDate", &input));
    track.released_orig_at = tag
        .take_strings(&ItemKey::OriginalReleaseDate)
        .find_map(|input| importer.import_year_tag_from_field("OriginalReleaseDate", &input));

    let mut tags_map: TagsMap<'static> = Default::default();

    // Grouping tags
    debug_assert!(tags_map.get_faceted_plain_tags(FACET_ID_GROUPING).is_none());
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_GROUPING,
        tag.take_strings(&compatibility.primary_content_group_item_key)
            .map(Into::into),
    );
    if let Some(secondary_content_group_item_key) = compatibility.secondary_content_group_item_key {
        if tags_map.get_faceted_plain_tags(FACET_ID_GROUPING).is_none() {
            importer.import_faceted_tags_from_label_values(
                &mut tags_map,
                &config.faceted_tag_mapping,
                FACET_ID_GROUPING,
                tag.take_strings(&secondary_content_group_item_key)
                    .map(Into::into),
            );
        }
    }

    // Import gig tags from raw grouping tags before any other tags.
    #[cfg(feature = "gigtag")]
    if config.flags.contains(ImportTrackFlags::GIGTAGS) {
        if let Some(faceted_tags) = tags_map.take_faceted_tags(FACET_ID_GROUPING) {
            tags_map = crate::util::gigtag::import_from_faceted_tags(faceted_tags);
        }
    }

    // Genre tags
    {
        let tag_mapping_config = config.faceted_tag_mapping.get(FACET_ID_GENRE.as_str());
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
        tags_map.update_faceted_plain_tags_by_label_ordering(FACET_ID_GENRE, plain_tags);
    }

    // Mood tags
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_MOOD,
        tag.take_strings(&ItemKey::Mood).map(Into::into),
    );

    // Comment tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_COMMENT,
        tag.take_strings(&ItemKey::Comment).map(Into::into),
    );

    // Description tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_DESCRIPTION,
        tag.take_strings(&ItemKey::Description).map(Into::into),
    );

    // ISRC tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_ISRC,
        tag.take_strings(&ItemKey::ISRC).map(Into::into),
    );

    // XID tag
    importer.import_faceted_tags_from_label_values(
        &mut tags_map,
        &config.faceted_tag_mapping,
        FACET_ID_XID,
        tag.take_strings(&ItemKey::AppleXid).map(Into::into),
    );

    debug_assert!(track.tags.is_empty());
    track.tags = Canonical::tie(tags_map.into());

    // Artwork
    if config
        .flags
        .contains(ImportTrackFlags::METADATA_EMBEDDED_ARTWORK)
    {
        let artwork = import_embedded_artwork(importer, &tag, config.flags.new_artwork_digest());
        track.media_source.artwork = Some(artwork);
    }
}

fn export_filtered_actor_names(
    tag: &mut Tag,
    item_key: ItemKey,
    actor_names: FilteredActorNames<'_>,
) {
    match actor_names {
        FilteredActorNames::Summary(name) => {
            tag.insert_text(item_key, name.to_owned());
        }
        FilteredActorNames::Individual(names) => {
            for name in names {
                let item_key = item_key.clone();
                let item_val = ItemValue::Text(name.to_owned());
                let pushed = tag.push_item(TagItem::new(item_key, item_val));
                if !pushed {
                    // Unsupported key
                    break;
                }
            }
        }
    }
}

fn export_faceted_tags(
    tag: &mut Tag,
    item_key: ItemKey,
    config: Option<&TagMappingConfig>,
    tags: impl IntoIterator<Item = PlainTag<'static>>,
) {
    if let Some(config) = config {
        let joined_labels = config.join_labels(
            tags.into_iter()
                .filter_map(|PlainTag { label, score: _ }| label.map(Label::into_inner)),
        );
        if let Some(joined_labels) = joined_labels {
            let inserted = tag.insert_text(item_key, joined_labels.into_owned());
            debug_assert!(inserted);
        } else {
            tag.remove_key(&item_key);
        }
    } else {
        tag.remove_key(&item_key);
        for label in tags
            .into_iter()
            .filter_map(|PlainTag { label, score: _ }| label)
        {
            let item_key = item_key.clone();
            let item_val = ItemValue::Text(label.into_inner().into_owned());
            let pushed = tag.push_item(TagItem::new(item_key, item_val));
            if !pushed {
                // Unsupported key
                break;
            }
        }
    }
}

#[allow(clippy::too_many_lines)] // TODO
pub(crate) fn export_track_to_tag(
    tag: &mut Tag,
    config: &ExportTrackConfig,
    track: &mut Track,
    replace_embedded_artwork_image: Option<ReplaceEmbeddedArtworkImage>,
) {
    // Audio properties
    match &track.media_source.content.metadata {
        ContentMetadata::Audio(audio) => {
            if let Some(track_gain_text) = audio.loudness.and_then(format_valid_replay_gain) {
                tag.insert_text(ItemKey::ReplayGainTrackGain, track_gain_text);
            } else {
                tag.remove_key(&ItemKey::ReplayGainTrackGain);
            }
            // The encoder is a read-only property.
        }
    }

    let compatibility = Compatibility::export(tag.tag_type(), config.flags);
    // Prevent inconsistencies by removing ambiguous keys.
    if let Some(secondary_content_group_item_key) = compatibility.secondary_content_group_item_key {
        tag.remove_key(&secondary_content_group_item_key);
    }
    if let Some(secondary_work_item_key) = compatibility.secondary_work_item_key {
        tag.remove_key(&secondary_work_item_key);
    }

    // Music: Tempo/BPM
    // Write the BPM rounded to an integer value as the least common denominator.
    // The precise BPM could be written into custom tag fields during post-processing.
    if let Some(bpm_text) =
        format_validated_tempo_bpm(&mut track.metrics.tempo_bpm, TempoBpmFormat::Integer)
    {
        tag.insert_text(ItemKey::BPM, bpm_text);
    } else {
        tag.remove_key(&ItemKey::BPM);
    }

    // Music: Key
    if let Some(key_signature) = track.metrics.key_signature {
        let key_text = key_signature_as_str(key_signature).to_owned();
        tag.insert_text(ItemKey::InitialKey, key_text);
    } else {
        tag.remove_key(&ItemKey::InitialKey);
    }

    // Track titles
    if let Some(title) = Titles::main_title(track.titles.iter()) {
        tag.set_title(title.name.clone());
    } else {
        tag.remove_title();
    }
    tag.remove_key(&ItemKey::TrackSubtitle);
    for track_subtitle in Titles::filter_kind(track.titles.iter(), TitleKind::Sub).peekable() {
        let item_val = ItemValue::Text(track_subtitle.name.clone());
        let pushed = tag.push_item(TagItem::new(ItemKey::TrackSubtitle, item_val));
        debug_assert!(pushed);
    }
    for movement_title in Titles::filter_kind(track.titles.iter(), TitleKind::Movement).peekable() {
        let item_val = ItemValue::Text(movement_title.name.clone());
        let pushed = tag.push_item(TagItem::new(ItemKey::Movement, item_val));
        debug_assert!(pushed);
    }
    for work_title in Titles::filter_kind(track.titles.iter(), TitleKind::Work).peekable() {
        let item_val = ItemValue::Text(work_title.name.clone());
        let pushed = tag.push_item(TagItem::new(
            compatibility.primary_work_item_key.clone(),
            item_val,
        ));
        debug_assert!(pushed);
    }

    // Track actors
    export_filtered_actor_names(
        tag,
        ItemKey::TrackArtist,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Artist),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Arranger,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Arranger),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Composer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Composer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Conductor,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Conductor),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Director,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Director),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Engineer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Engineer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::MixDj,
        FilteredActorNames::new(track.actors.iter(), ActorRole::MixDj),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::MixEngineer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::MixEngineer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Lyricist,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Lyricist),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Performer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Performer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Producer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Producer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Remixer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Remixer),
    );
    export_filtered_actor_names(
        tag,
        ItemKey::Writer,
        FilteredActorNames::new(track.actors.iter(), ActorRole::Writer),
    );

    // Album
    if let Some(title) = Titles::main_title(track.album.titles.iter()) {
        tag.set_album(title.name.clone());
    } else {
        tag.remove_album();
    }
    export_filtered_actor_names(
        tag,
        ItemKey::AlbumArtist,
        FilteredActorNames::new(track.album.actors.iter(), ActorRole::Artist),
    );
    if let Some(kind) = track.album.kind {
        match kind {
            AlbumKind::NoCompilation | AlbumKind::Album | AlbumKind::Single => {
                tag.insert_text(ItemKey::FlagCompilation, "0".to_owned());
            }
            AlbumKind::Compilation => {
                tag.insert_text(ItemKey::FlagCompilation, "1".to_owned());
            }
        }
    } else {
        tag.remove_key(&ItemKey::FlagCompilation);
    }

    if let Some(publisher) = &track.publisher {
        tag.insert_text(ItemKey::Label, publisher.clone());
    } else {
        tag.remove_key(&ItemKey::Label);
    }
    if let Some(copyright) = &track.copyright {
        tag.insert_text(ItemKey::CopyrightMessage, copyright.clone());
    } else {
        tag.remove_key(&ItemKey::CopyrightMessage);
    }

    // Index pairs
    if let Some(track_number) = track.indexes.track.number {
        tag.set_track(track_number.into());
    } else {
        tag.remove_track();
    }
    if let Some(track_total) = track.indexes.track.total {
        tag.set_track_total(track_total.into());
    } else {
        tag.remove_track_total();
    }
    if let Some(disc_number) = track.indexes.disc.number {
        tag.set_disk(disc_number.into());
    } else {
        tag.remove_disk();
    }
    if let Some(disc_total) = track.indexes.disc.total {
        tag.set_disk_total(disc_total.into());
    } else {
        tag.remove_disk_total();
    }
    if let Some(movement_number) = track.indexes.movement.number {
        tag.insert_text(ItemKey::MovementNumber, movement_number.to_string());
    } else {
        tag.remove_key(&ItemKey::MovementNumber);
    }
    if let Some(movement_total) = track.indexes.movement.total {
        tag.insert_text(ItemKey::MovementTotal, movement_total.to_string());
    } else {
        tag.remove_key(&ItemKey::MovementTotal);
    }

    if let Some(recorded_at) = track.recorded_at {
        // Modify ItemKey::Year before ItemKey::RecordingDate
        // because they may end up in the same tag field. The
        // year number should not overwrite the more specific
        // time stamp if available.
        let year = recorded_at.year();
        if year >= 0 {
            tag.set_year(year as _);
        } else {
            tag.remove_year();
        }
        let recorded_at_text = recorded_at.to_string();
        tag.insert_text(ItemKey::RecordingDate, recorded_at_text);
    } else {
        tag.remove_year();
        tag.remove_key(&ItemKey::RecordingDate);
    }
    if let Some(released_at) = track.released_at {
        let released_at_text = released_at.to_string();
        tag.insert_text(ItemKey::PodcastReleaseDate, released_at_text);
    } else {
        tag.remove_key(&ItemKey::PodcastReleaseDate);
    }
    if let Some(released_orig_at) = track.released_orig_at {
        let released_orig_at_text = released_orig_at.to_string();
        tag.insert_text(ItemKey::OriginalReleaseDate, released_orig_at_text);
    } else {
        tag.remove_key(&ItemKey::OriginalReleaseDate);
    }

    let mut tags_map = TagsMap::from(track.tags.clone().untie());

    // Genre(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_GENRE) {
        export_faceted_tags(
            tag,
            ItemKey::Genre,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags.into_iter(),
        );
    } else {
        tag.remove_key(&ItemKey::Genre);
    }

    // Comment(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_COMMENT) {
        export_faceted_tags(
            tag,
            ItemKey::Comment,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::Comment);
    }

    // Description(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_DESCRIPTION) {
        export_faceted_tags(
            tag,
            ItemKey::Description,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::Description);
    }

    // Mood(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_MOOD) {
        export_faceted_tags(
            tag,
            ItemKey::Mood,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::Mood);
    }

    // ISRC(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_ISRC) {
        export_faceted_tags(
            tag,
            ItemKey::ISRC,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::ISRC);
    }

    // XID(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_XID) {
        export_faceted_tags(
            tag,
            ItemKey::AppleXid,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        tag.remove_key(&ItemKey::AppleXid);
    }

    // Grouping(s)
    {
        let facet_id = FACET_ID_GROUPING;
        let mut tags = tags_map
            .take_faceted_tags(facet_id)
            .map(|FacetedTags { facet_id: _, tags }| tags)
            .unwrap_or_default();
        #[cfg(feature = "gigtag")]
        if config.flags.contains(ExportTrackFlags::GIGTAGS) {
            if let Err(err) = crate::util::gigtag::export_and_encode_remaining_tags_into(
                tags_map.into(),
                &mut tags,
            ) {
                log::error!("Failed to export gig tags: {err}");
            }
        }
        if tags.is_empty() {
            tag.remove_key(&compatibility.primary_content_group_item_key);
        } else {
            export_faceted_tags(
                tag,
                compatibility.primary_content_group_item_key,
                config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
                tags,
            );
        }
    }

    if let Some(replace_embedded_artwork_image) = replace_embedded_artwork_image {
        let ReplaceEmbeddedArtworkImage {
            artwork_image,
            image_data,
            others,
        } = replace_embedded_artwork_image;
        let ArtworkImage {
            apic_type,
            media_type,
            ..
        } = &artwork_image;
        let pic_type = picture_type_from_apic_type(*apic_type);
        let mime_type = match media_type.essence_str() {
            "image/bmp" => MimeType::Bmp,
            "image/gif" => MimeType::Gif,
            "image/jpeg" => MimeType::Jpeg,
            "image/png" => MimeType::Png,
            "image/tiff" => MimeType::Tiff,
            _ => MimeType::Unknown(media_type.to_string()),
        };
        track.media_source.artwork = Some(Artwork::Embedded(EmbeddedArtwork {
            image: artwork_image,
        }));
        let picture = Picture::new_unchecked(pic_type, mime_type, None, image_data);
        match others {
            ReplaceOtherEmbeddedArtworkImages::Keep => {
                tag.remove_picture_type(pic_type);
            }
            ReplaceOtherEmbeddedArtworkImages::Remove => {
                while !tag.pictures().is_empty() {
                    tag.remove_picture(tag.pictures().len() - 1);
                }
            }
        }
        tag.push_picture(picture);
    }
}
