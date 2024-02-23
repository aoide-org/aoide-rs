// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{path::PathBuf, sync::mpsc};

use eframe::{CreationContext, Frame};
use egui::{Color32, ColorImage, Context, TextureHandle, TextureOptions};

use aoide::{
    desktop_app::fs::DirPath,
    media::{
        artwork::{Artwork, ArtworkImage, EmbeddedArtwork},
        content::ContentPath,
    },
    music::{key::KeySignature, tempo::TempoBpm},
    tag::FacetId,
    track::tag::{FACET_ID_COMMENT, FACET_ID_GENRE},
    util::{clock::DateOrDateTime, color::RgbColor},
    TrackUid,
};

use crate::{
    library::{self, track_search, Library},
    NoReceiverForEvent,
};

mod update;
use self::update::UpdateContext;

mod render;
use self::render::RenderContext;

#[allow(missing_debug_implementations)]
struct NoReceiverForMessage(Message);

#[allow(missing_debug_implementations)]
#[derive(Clone)]
struct MessageSender {
    ctx: Context,
    msg_tx: mpsc::Sender<Message>,
}

impl MessageSender {
    const fn new(ctx: Context, msg_tx: mpsc::Sender<Message>) -> Self {
        Self { ctx, msg_tx }
    }

    fn send_action<T>(&self, action: T)
    where
        T: Into<Action>,
    {
        if let Err(NoReceiverForMessage(msg)) = self.send_message(Message::Action(action.into())) {
            let Message::Action(_) = msg else {
                unreachable!()
            };
            log::warn!("No receiver for action");
        }
    }

    #[allow(dead_code)] // TODO: Currently unused?
    fn emit_event<T>(&self, event: T) -> Result<(), NoReceiverForEvent>
    where
        T: Into<Event>,
    {
        if let Err(NoReceiverForMessage(msg)) = self.send_message(Message::Event(event.into())) {
            let Message::Event(event) = msg else {
                unreachable!()
            };
            log::warn!("No receiver for event: {event:?}");
            return Err(NoReceiverForEvent);
        }
        Ok(())
    }

    fn send_message(&self, msg: Message) -> Result<(), NoReceiverForMessage> {
        self.msg_tx.send(msg).map_err(|err| {
            log::warn!("Failed to send message: {err}");
            NoReceiverForMessage(err.0)
        })?;
        // Queued messages are consumed before rendering the next frame.
        self.ctx.request_repaint();
        Ok(())
    }
}

impl library::EventEmitter for MessageSender {
    fn emit_event(&self, event: library::Event) -> Result<(), NoReceiverForEvent> {
        let event: Event = Event::Library(event);
        self.send_message(Message::Event(event))
            .map_err(|NoReceiverForMessage(_)| NoReceiverForEvent)
    }
}

// Not cloneable so large enum variants should be fine.
#[allow(clippy::large_enum_variant)]
#[allow(missing_debug_implementations)]
enum Message {
    Action(Action),
    Event(Event),
}

#[allow(missing_debug_implementations)]
enum Action {
    Library(LibraryAction),
}

impl From<Action> for Message {
    fn from(action: Action) -> Self {
        Self::Action(action)
    }
}

#[allow(missing_debug_implementations)]
enum LibraryAction {
    MusicDirectory(MusicDirectoryAction),
    Collection(CollectionAction),
    TrackSearch(TrackSearchAction),
}

impl<T> From<T> for Action
where
    T: Into<LibraryAction>,
{
    fn from(action: T) -> Self {
        Self::Library(action.into())
    }
}

#[derive(Debug)]
enum MusicDirectoryAction {
    Reset,
    Select,
    Update(Option<DirPath<'static>>),
    SpawnSyncTask,
    AbortPendingSyncTask,
    ViewList,
}

impl From<MusicDirectoryAction> for LibraryAction {
    fn from(action: MusicDirectoryAction) -> Self {
        Self::MusicDirectory(action)
    }
}

#[derive(Debug, Clone)]
enum CollectionAction {
    RefreshFromDb,
}

impl From<CollectionAction> for LibraryAction {
    fn from(action: CollectionAction) -> Self {
        Self::Collection(action)
    }
}

#[allow(missing_debug_implementations)]
enum TrackSearchAction {
    Search(String),
    FetchMore,
    UpdateStateAndList {
        memo: track_search::Memo,
        memo_delta: track_search::MemoDelta,
        fetched_entities_diff: track_search::FetchedEntitiesDiff,
        fetched_items: Option<Vec<TrackListItem>>,
    },
}

impl From<TrackSearchAction> for LibraryAction {
    fn from(action: TrackSearchAction) -> Self {
        Self::TrackSearch(action)
    }
}

/// App-level event
///
/// Not cloneable to prevent unintended storage. Notifications are
/// supposed to be ephemeral and should disappear after being processed.
#[derive(Debug)]
enum Event {
    Library(library::Event),
}

impl From<Event> for Message {
    fn from(event: Event) -> Self {
        Self::Event(event)
    }
}

enum CentralPanelData {
    TrackSearch {
        track_list: Vec<TrackListItem>,
    },
    MusicDirSync {
        progress_log: Vec<String>,
    },
    MusicDirList {
        content_paths_with_count: Vec<(ContentPath<'static>, usize)>,
    },
}

#[derive(Debug)]
enum MusicDirSelection {
    Selecting,
    Selected,
}

/// Application model
///
/// Immutable during rendering.
#[allow(missing_debug_implementations)]
pub struct Model {
    library: Library,

    music_dir_selection: Option<MusicDirSelection>,

    central_panel_data: Option<CentralPanelData>,
}

impl Model {
    #[must_use]
    pub const fn new(library: Library) -> Self {
        Self {
            library,
            music_dir_selection: None,
            central_panel_data: None,
        }
    }
}

/// UI data bindings
///
/// Stores user input and other mutable UI state that needs to be preserved
/// between frames.
#[derive(Debug, Default)]
struct UiData {
    track_search_input: String,
}

#[allow(missing_debug_implementations)]
pub struct App {
    rt: tokio::runtime::Handle,

    msg_rx: mpsc::Receiver<Message>,
    msg_tx: MessageSender,

    mdl: Model,

    ui_data: UiData,
}

impl App {
    #[must_use]
    pub fn new(
        ctx: &CreationContext<'_>,
        rt: tokio::runtime::Handle,
        mdl: Model,
        settings_dir: PathBuf,
    ) -> Self {
        let (msg_tx, msg_rx) = mpsc::channel();
        let msg_tx = MessageSender::new(ctx.egui_ctx.clone(), msg_tx);
        mdl.library.spawn_background_tasks(&rt, settings_dir);
        mdl.library.spawn_event_tasks(&rt, &msg_tx);
        Self {
            rt,
            msg_rx,
            msg_tx,
            mdl,
            ui_data: Default::default(),
        }
    }

    fn update(&mut self) -> (&mpsc::Receiver<Message>, UpdateContext<'_>) {
        let Self {
            rt,
            msg_rx,
            msg_tx,
            mdl,
            ..
        } = self;
        let ctx = UpdateContext { rt, msg_tx, mdl };
        (msg_rx, ctx)
    }

    fn render(&mut self) -> RenderContext<'_> {
        let Self {
            msg_tx,
            mdl,
            ui_data,
            ..
        } = self;
        RenderContext {
            msg_tx,
            mdl,
            ui_data,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frm: &mut Frame) {
        let (msg_rx, mut update_ctx) = self.update();
        let msg_count = msg_rx
            .try_iter()
            .map(|msg| {
                update_ctx.on_message(ctx, msg);
            })
            .count();
        if msg_count > 0 {
            log::debug!("Processed {msg_count} message(s) before rendering frame");
        }

        let mut render_ctx = self.render();
        render_ctx.render_frame(ctx, frm);
    }
}

/// Simplified, pre-rendered track data
#[allow(missing_debug_implementations)]
pub struct TrackListItem {
    pub entity_uid: TrackUid,
    pub artwork_thumbnail_texture: TextureHandle,

    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_title: Option<String>,
    pub album_artist: Option<String>,
    pub genres: Vec<String>,
    pub comments: Vec<String>,
    pub year_min: Option<i16>,
    pub year_max: Option<i16>,
    pub bpm: Option<TempoBpm>,
    pub key: Option<KeySignature>,
}

impl TrackListItem {
    #[must_use]
    pub fn new(ctx: &Context, entity_uid: aoide::TrackUid, track: &aoide::Track) -> Self {
        let artwork_thumbnail_image = track
            .media_source
            .artwork
            .as_ref()
            .and_then(|artwork| {
                let default_color = match track.color {
                    Some(aoide::util::color::Color::Rgb(rgb_color)) => Some(rgb_color),
                    _ => None,
                };
                artwork_thumbnail_image(artwork, default_color)
            })
            .unwrap_or_else(|| {
                // TODO: Use a single, shared, transparent texture for all tracks without artwork.
                artwork_thumbnail_image_placeholder()
            });
        // TODO: Only load the texture once for each distinct image -> hash the image data.
        let artwork_thumbnail_texture = ctx.load_texture(
            "", // anonymous
            artwork_thumbnail_image,
            TextureOptions::LINEAR,
        );
        let artist = track.track_artist().map(ToOwned::to_owned);
        let title = track.track_title().map(ToOwned::to_owned);
        let album_title = track.album_title().map(ToOwned::to_owned);
        let album_artist = track.album_artist().map(ToOwned::to_owned);
        let genres = filter_faceted_tag_labels(track, FACET_ID_GENRE)
            .map(ToString::to_string)
            .collect();
        let comments = filter_faceted_tag_labels(track, FACET_ID_COMMENT)
            .map(ToString::to_string)
            .collect();
        let dates = track
            .recorded_at
            .into_iter()
            .chain(track.released_at)
            .chain(track.released_orig_at);
        let year_min = dates.clone().map(DateOrDateTime::year).min();
        let year_max = dates.map(DateOrDateTime::year).max();
        let bpm = track.metrics.tempo_bpm;
        let key = track.metrics.key_signature;
        Self {
            entity_uid,
            artwork_thumbnail_texture,
            title,
            artist,
            album_title,
            album_artist,
            genres,
            comments,
            year_min,
            year_max,
            bpm,
            key,
        }
    }
}

fn filter_faceted_tag_labels<'a>(
    track: &'a aoide::Track,
    facet_id: &'a FacetId<'a>,
) -> impl Iterator<Item = &'a aoide::tag::Label<'a>> {
    track
        .tags
        .facets
        .iter()
        .filter_map(|faceted_tags| {
            if faceted_tags.facet_id == *facet_id {
                Some(faceted_tags.tags.iter())
            } else {
                None
            }
        })
        .flatten()
        .filter_map(|tag| tag.label.as_ref())
}

const ARTWORK_THUMBNAIL_SIZE: usize = 4;
const ARTWORK_THUMBNAIL_IMAGE_SIZE: usize = 6;
const ARTWORK_THUMBNAIL_BORDER_SIZE: usize = 1;

#[must_use]
const fn solid_rgb_color(color: RgbColor) -> Color32 {
    Color32::from_rgb(color.red(), color.green(), color.blue())
}

#[must_use]
fn artwork_thumbnail_image_with_solid_color(color: Color32) -> ColorImage {
    ColorImage {
        size: [ARTWORK_THUMBNAIL_IMAGE_SIZE, ARTWORK_THUMBNAIL_IMAGE_SIZE],
        pixels: [color; ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_IMAGE_SIZE].to_vec(),
    }
}

#[must_use]
fn artwork_thumbnail_image_placeholder() -> ColorImage {
    artwork_thumbnail_image_with_solid_color(Color32::TRANSPARENT)
}

#[must_use]
fn artwork_thumbnail_image_from_rgb_pixels(
    thumbnail: &[u8; ARTWORK_THUMBNAIL_SIZE * ARTWORK_THUMBNAIL_SIZE * 3],
    border_color: Color32,
) -> ColorImage {
    let pixels = thumbnail
        .chunks_exact(3)
        .map(|rgb| Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
    artwork_thumbnail_image_from_pixels(pixels, border_color)
}

#[must_use]
#[allow(clippy::similar_names)]
fn artwork_thumbnail_image_from_pixels(
    pixels: impl IntoIterator<Item = Color32>,
    border_color: Color32,
) -> ColorImage {
    // TODO: Avoid temporary allocation.
    let pixels = pixels.into_iter().collect::<Vec<_>>();
    let mut pixels_rows = pixels.chunks_exact(4);
    let pixels_row0 = pixels_rows.next().unwrap();
    let pixels_row1 = pixels_rows.next().unwrap();
    let pixels_row2 = pixels_rows.next().unwrap();
    let pixels_row3 = pixels_rows.next().unwrap();
    let pixels = std::iter::repeat(border_color)
        .take(
            ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_BORDER_SIZE
                + ARTWORK_THUMBNAIL_BORDER_SIZE,
        )
        .chain(pixels_row0.iter().copied())
        .chain(std::iter::repeat(border_color).take(ARTWORK_THUMBNAIL_BORDER_SIZE * 2))
        .chain(pixels_row1.iter().copied())
        .chain(std::iter::repeat(border_color).take(ARTWORK_THUMBNAIL_BORDER_SIZE * 2))
        .chain(pixels_row2.iter().copied())
        .chain(std::iter::repeat(border_color).take(ARTWORK_THUMBNAIL_BORDER_SIZE * 2))
        .chain(pixels_row3.iter().copied())
        .chain(std::iter::repeat(border_color).take(
            ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_BORDER_SIZE
                + ARTWORK_THUMBNAIL_BORDER_SIZE,
        ))
        .collect::<Vec<_>>();
    debug_assert_eq!(
        pixels.len(),
        ARTWORK_THUMBNAIL_IMAGE_SIZE * ARTWORK_THUMBNAIL_IMAGE_SIZE
    );
    ColorImage {
        size: [ARTWORK_THUMBNAIL_IMAGE_SIZE, ARTWORK_THUMBNAIL_IMAGE_SIZE],
        pixels,
    }
}

#[must_use]
#[allow(clippy::similar_names)]
fn artwork_thumbnail_image(
    artwork: &Artwork,
    default_color: Option<RgbColor>,
) -> Option<ColorImage> {
    let Artwork::Embedded(EmbeddedArtwork {
        image: ArtworkImage {
            thumbnail, color, ..
        },
        ..
    }) = artwork
    else {
        return None;
    };
    let color = color.or(default_color);
    let Some(thumbnail) = thumbnail else {
        return color
            .map(solid_rgb_color)
            .map(artwork_thumbnail_image_with_solid_color);
    };
    let color = color.map(solid_rgb_color);
    let border_color = color.unwrap_or(Color32::TRANSPARENT);
    Some(artwork_thumbnail_image_from_rgb_pixels(
        thumbnail,
        border_color,
    ))
}
