// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{path::PathBuf, sync::mpsc};

use eframe::{CreationContext, Frame};
use egui::{Color32, ColorImage, Context};

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
};

use crate::{
    library::{self, Library},
    NoReceiverForEvent,
};

mod update;
use self::update::UpdateContext;

mod render;
use self::render::RenderContext;

#[derive(Debug)]
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
            let Message::Action(action) = msg else {
                unreachable!()
            };
            log::warn!("No receiver for action: {action:?}");
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
        log::debug!("Sending message {msg:?}");
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

#[derive(Debug)]
// Not cloneable so large enum variants should be fine.
#[allow(clippy::large_enum_variant)]
enum Message {
    Action(Action),
    Event(Event),
}

#[derive(Debug)]
enum Action {
    Library(LibraryAction),
}

impl From<Action> for Message {
    fn from(action: Action) -> Self {
        Self::Action(action)
    }
}

#[derive(Debug)]
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

#[derive(Debug, Clone)]
enum TrackSearchAction {
    Search(String),
    FetchMore,
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
        track_list: Vec<Track>,
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
                log::debug!("Received message: {msg:?}");
                update_ctx.on_message(msg);
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
#[derive(Debug)]
pub struct Track {
    pub artwork_thumbnail: Option<ColorImage>,
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

impl Track {
    #[must_use]
    pub fn new(track: &aoide::Track) -> Self {
        let artwork_thumbnail = track
            .media_source
            .artwork
            .as_ref()
            .and_then(|artwork| artwork_thumbnail_image(artwork, true))
            .or_else(|| {
                // Fallback: Use the track color if no artwork is available.
                track.color.and_then(|color| {
                    let aoide::util::color::Color::Rgb(rgb_color) = color else {
                        return None;
                    };
                    Some(artwork_thumbnail_image_with_solid_color(
                        solid_rgb_color(rgb_color),
                        true,
                    ))
                })
            });
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
            artwork_thumbnail,
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

#[must_use]
const fn solid_rgb_color(color: RgbColor) -> Color32 {
    Color32::from_rgb(color.red(), color.green(), color.blue())
}

#[must_use]
fn artwork_thumbnail_image_with_solid_color(color: Color32, with_border: bool) -> ColorImage {
    if with_border {
        let pixels = [color; 6 * 6].to_vec();
        ColorImage {
            size: [6; 2],
            pixels,
        }
    } else {
        let pixels = [color; 4 * 4].to_vec();
        ColorImage {
            size: [4; 2],
            pixels,
        }
    }
}

#[must_use]
#[allow(clippy::similar_names)]
fn artwork_thumbnail_image(artwork: &Artwork, with_border: bool) -> Option<ColorImage> {
    let Artwork::Embedded(EmbeddedArtwork {
        image:
            ArtworkImage {
                thumbnail: rgb_4x4,
                color,
                ..
            },
        ..
    }) = artwork
    else {
        return None;
    };
    let color = color.map(solid_rgb_color);
    let Some(rgb_4x4) = rgb_4x4 else {
        return color.map(|color| artwork_thumbnail_image_with_solid_color(color, with_border));
    };
    let thumbnail_pixels = rgb_4x4
        .chunks_exact(3)
        .map(|rgb| Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
    let image = if with_border && color.is_some() {
        let color = color.unwrap();
        // TODO: Avoid temporary allocation.
        let thumbnail_pixels = thumbnail_pixels.collect::<Vec<_>>();
        let mut thumbnail_pixels_rows = thumbnail_pixels.chunks_exact(4);
        let thumbnail_pixels_row0 = thumbnail_pixels_rows.next().unwrap();
        let thumbnail_pixels_row1 = thumbnail_pixels_rows.next().unwrap();
        let thumbnail_pixels_row2 = thumbnail_pixels_rows.next().unwrap();
        let thumbnail_pixels_row3 = thumbnail_pixels_rows.next().unwrap();
        let pixels_6x6 = [color; 7]
            .into_iter()
            .chain(thumbnail_pixels_row0.iter().copied())
            .chain([color; 2])
            .chain(thumbnail_pixels_row1.iter().copied())
            .chain([color; 2])
            .chain(thumbnail_pixels_row2.iter().copied())
            .chain([color; 2])
            .chain(thumbnail_pixels_row3.iter().copied())
            .chain([color; 7])
            .collect::<Vec<_>>();
        debug_assert_eq!(pixels_6x6.len(), 6 * 6);
        ColorImage {
            size: [6, 6],
            pixels: pixels_6x6,
        }
    } else {
        let pixels_4x4 = thumbnail_pixels.collect::<Vec<_>>();
        debug_assert_eq!(pixels_4x4.len(), 4 * 4);
        ColorImage {
            size: [4, 4],
            pixels: pixels_4x4,
        }
    };
    Some(image)
}
