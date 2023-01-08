// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{iter::once, path::Path};

use mp4ameta::{Data, DataIdent, Fourcc, FreeformIdent, Ident, Tag as Mp4Tag};

use aoide_core::{
    media::content::ContentMetadata,
    music::tempo::TempoBpm,
    tag::{FacetKey, FacetedTags, Label, PlainTag, TagsMap},
    track::{
        actor::Role as ActorRole,
        album::Kind as AlbumKind,
        tag::{
            FACET_ID_COMMENT, FACET_ID_DESCRIPTION, FACET_ID_GENRE, FACET_ID_GROUPING,
            FACET_ID_ISRC, FACET_ID_MOOD, FACET_ID_XID,
        },
        title::{Kind as TitleKind, Titles},
        Track,
    },
};

use crate::{
    io::export::{ExportTrackConfig, ExportTrackFlags, FilteredActorNames},
    util::{
        format_valid_replay_gain, format_validated_tempo_bpm, key_signature_as_str,
        tag::TagMappingConfig,
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

const COM_APPLE_ITUNES_FREEFORM_MEAN: &str = "com.apple.iTunes";

const IDENT_ALBUM_ARTIST: Fourcc = Fourcc(*b"aART");

const IDENT_ARTIST: Fourcc = Fourcc(*b"\xA9ART");

const IDENT_COMMENT: Fourcc = Fourcc(*b"\xA9cmt");

const IDENT_COMPILATION: Fourcc = Fourcc(*b"cpil");

const IDENT_COMPOSER: Fourcc = Fourcc(*b"\xA9wrt");

const IDENT_DESCRIPTION: Fourcc = Fourcc(*b"desc");

const IDENT_DIRECTOR: Fourcc = Fourcc(*b"\xA9dir");

const IDENT_GENRE: Fourcc = Fourcc(*b"\xA9gen");

const IDENT_GROUPING: Fourcc = Fourcc(*b"\xA9grp");

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
                    .filter_map(|PlainTag { label, score: _ }| label.as_ref().map(Label::as_str)),
            )
            .clone();
        mp4_tag.set_all_data(ident, joined_labels.map(|s| Data::Utf8(s.into())));
    } else {
        mp4_tag.set_all_data(
            ident,
            tags.into_iter().filter_map(|PlainTag { label, score: _ }| {
                label.map(|label| Data::Utf8(label.into_inner().into_owned()))
            }),
        );
    }
}

#[allow(clippy::too_many_lines)] // TODO
pub(crate) fn export_track_to_path(
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
            // The encoder is a read-only property.
        }
    }

    // Music: Tempo/BPM
    if let Some(formatted_bpm) = format_validated_tempo_bpm(&mut track.metrics.tempo_bpm) {
        mp4_tag.set_all_data(IDENT_BPM, once(Data::Utf8(formatted_bpm)));
        #[allow(clippy::cast_possible_truncation)]
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
    if let Some(key_signature) = track.metrics.key_signature {
        let key_data = Data::Utf8(key_signature_as_str(key_signature).into());
        if mp4_tag.data_of(&KEY_IDENT).next().is_some() {
            // Write non-standard key atom only if already present
            mp4_tag.set_all_data(KEY_IDENT, once(key_data.clone()));
        }
        mp4_tag.set_all_data(IDENT_INITIAL_KEY, once(key_data));
    } else {
        mp4_tag.remove_data_of(&IDENT_INITIAL_KEY);
        mp4_tag.remove_data_of(&KEY_IDENT);
    }

    // Track titles
    if let Some(title) = Titles::main_title(track.titles.iter()) {
        mp4_tag.set_title(title.name.clone());
    } else {
        mp4_tag.remove_title();
    }
    let track_subtitles = Titles::filter_kind(track.titles.iter(), TitleKind::Sub).peekable();
    mp4_tag.set_all_data(
        IDENT_SUBTITLE,
        track_subtitles.map(|subtitle| Data::Utf8(subtitle.name.clone())),
    );
    let mut track_movements =
        Titles::filter_kind(track.titles.iter(), TitleKind::Movement).peekable();
    if track_movements.peek().is_some() {
        let movement = track_movements.next().unwrap();
        // Only a single movement is supported
        debug_assert!(track_movements.peek().is_none());
        mp4_tag.set_movement(movement.name.clone());
    } else {
        mp4_tag.remove_movement();
    }
    let mut track_works = Titles::filter_kind(track.titles.iter(), TitleKind::Work).peekable();
    if track_works.peek().is_some() {
        let work = track_works.next().unwrap();
        // Only a single work is supported
        debug_assert!(track_works.peek().is_none());
        mp4_tag.set_work(work.name.clone());
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
        FilteredActorNames::new(track.actors.iter(), ActorRole::MixEngineer),
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
        mp4_tag.set_album(title.name.clone());
    } else {
        mp4_tag.remove_album();
    }
    export_filtered_actor_names(
        &mut mp4_tag,
        IDENT_ALBUM_ARTIST,
        FilteredActorNames::new(track.album.actors.iter(), ActorRole::Artist),
    );
    if let Some(kind) = track.album.kind {
        match kind {
            AlbumKind::NoCompilation | AlbumKind::Album | AlbumKind::Single => {
                mp4_tag.set_data(IDENT_COMPILATION, Data::BeSigned(vec![0u8]));
            }
            AlbumKind::Compilation => {
                mp4_tag.set_data(IDENT_COMPILATION, Data::BeSigned(vec![1u8]));
            }
        }
    } else {
        mp4_tag.remove_data_of(&IDENT_COMPILATION);
    }

    // No distinction between recording and release date, i.e.
    // only the release date is stored.
    if let Some(recorded_at) = track.recorded_at {
        mp4_tag.set_year(recorded_at.to_string());
    } else {
        mp4_tag.remove_year();
    }
    if let Some(publisher) = &track.publisher {
        mp4_tag.set_all_data(IDENT_LABEL, once(Data::Utf8(publisher.clone())));
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
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_GENRE) {
        // Overwrite standard genres with custom genres
        mp4_tag.remove_standard_genres();
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_GENRE,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        // Preserve standard genres until overwritten by custom genres
        mp4_tag.remove_data_of(&IDENT_GENRE);
    }

    // Comment(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_COMMENT) {
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_COMMENT,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        mp4_tag.remove_data_of(&IDENT_COMMENT);
    }

    // Description(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_DESCRIPTION) {
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_DESCRIPTION,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        mp4_tag.remove_data_of(&IDENT_DESCRIPTION);
    }

    // Mood(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_MOOD) {
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_MOOD,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        mp4_tag.remove_data_of(&IDENT_MOOD);
    }

    // ISRC(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_ISRC) {
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_ISRC,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        mp4_tag.remove_data_of(&IDENT_ISRC);
    }

    // XID(s)
    if let Some(FacetedTags { facet_id, tags }) = tags_map.take_faceted_tags(FACET_ID_XID) {
        export_faceted_tags(
            &mut mp4_tag,
            IDENT_XID,
            config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
            tags,
        );
    } else {
        mp4_tag.remove_data_of(&IDENT_XID);
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
                log::error!("Failed to export gigitags: {err}");
            }
        }
        if tags.is_empty() {
            mp4_tag.remove_data_of(&IDENT_GROUPING);
        } else {
            export_faceted_tags(
                &mut mp4_tag,
                IDENT_GROUPING,
                config.faceted_tag_mapping.get(&FacetKey::from(facet_id)),
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
