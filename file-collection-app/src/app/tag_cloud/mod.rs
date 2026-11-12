// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Editing the plain tags of tracks with the help of a tag cloud.
//!
//! Tags are added by dragging them from the cloud onto the tags of a track
//! and removed by dragging them from a track back into the cloud.

use std::collections::BTreeSet;

use egui::{
    Align, Align2, Button, Context, CornerRadius, CursorIcon, DragAndDrop, FontId, Id, Key,
    LayerId, Layout, Order, Rect, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, TextStyle, Ui,
    Vec2, Window,
};
use itertools::Itertools as _;

use aoide::TrackUid;

use crate::library::ui::{CustomTagsDelta, MultiTrackEdit, MultiTrackFields, TrackListItem};

use super::{
    MessageSender, TagCloudAction, TrackEditAction, TrackSelection,
    render::help_icon,
    track_list::{paint_bubble, paint_bubble_with_font},
};

const WINDOW_ID: &str = "tag-cloud";

const WINDOW_DEFAULT_WIDTH: f32 = 360.0;

const WINDOW_DEFAULT_POS: [f32; 2] = [480.0, 120.0];

/// Relative font size of the least frequent tags.
const MIN_FONT_SCALE: f32 = 0.85;

/// Relative font size of the most frequent tags.
const MAX_FONT_SCALE: f32 = 1.6;

const DROP_TARGET_STROKE_WIDTH: f32 = 2.0;

const EDITED_CELL_STROKE_WIDTH: f32 = 1.0;

const DRAGGED_TAGS_MARGIN: f32 = 4.0;

const DRAGGED_TAGS_POINTER_OFFSET: Vec2 = Vec2::new(16.0, 0.0);

/// The maximum number of labels that are displayed while dragging.
const DRAGGED_TAGS_MAX_LABELS: usize = 5;

/// The plain tags of the collection for editing the tags of a track.
#[derive(Debug)]
pub(crate) struct TagCloud {
    /// The track whose tags are edited.
    pub(crate) entity_uid: TrackUid,
    /// All labels with the number of tracks in ascending order.
    label_counts: Vec<(String, usize)>,
    load_error: Option<String>,
    pending_request_id: Option<u64>,
    last_request_id: u64,
}

impl TagCloud {
    #[must_use]
    pub(crate) const fn new(entity_uid: TrackUid) -> Self {
        Self {
            entity_uid,
            label_counts: Vec::new(),
            load_error: None,
            pending_request_id: None,
            last_request_id: 0,
        }
    }

    /// Start (re-)loading the tags.
    ///
    /// The current tags remain available until the result arrives.
    ///
    /// Returns the id of the request.
    pub(crate) fn start_loading(&mut self) -> u64 {
        self.last_request_id += 1;
        self.pending_request_id = Some(self.last_request_id);
        self.last_request_id
    }

    /// The request could not be sent.
    pub(crate) fn abort_loading(&mut self) {
        self.pending_request_id = None;
    }

    /// Results of outdated requests are discarded.
    ///
    /// Returns `true` if the result has been accepted.
    pub(crate) fn finish_loading(
        &mut self,
        request_id: u64,
        result: Result<Vec<(String, usize)>, String>,
    ) -> bool {
        if self.pending_request_id != Some(request_id) {
            return false;
        }
        self.pending_request_id = None;
        match result {
            Ok(label_counts) => {
                self.label_counts = label_counts;
                self.load_error = None;
            }
            Err(err) => {
                self.load_error = Some(err);
            }
        }
        true
    }

    #[must_use]
    pub(crate) const fn is_loading(&self) -> bool {
        self.pending_request_id.is_some()
    }

    #[must_use]
    fn contains(&self, label: &str) -> bool {
        self.label_counts
            .binary_search_by(|(other, _)| other.as_str().cmp(label))
            .is_ok()
    }
}

/// The relative font size of a tag depending on its frequency.
#[must_use]
#[expect(clippy::cast_precision_loss)]
fn font_scale(count: usize, max_count: usize) -> f32 {
    if max_count <= 1 {
        return MIN_FONT_SCALE;
    }
    // Logarithmic scale, otherwise a few frequent tags would dwarf all others.
    let ratio = (count.max(1) as f32).ln() / (max_count as f32).ln();
    MIN_FONT_SCALE + (MAX_FONT_SCALE - MIN_FONT_SCALE) * ratio.clamp(0.0, 1.0)
}

/// Selected tags and user input that is preserved between frames.
#[derive(Debug, Default)]
pub(super) struct TagSelection {
    /// The track that the selected track tags belong to.
    entity_uid: Option<TrackUid>,
    /// Selected labels in the cloud.
    cloud: BTreeSet<String>,
    /// Selected labels of the edited track.
    track: BTreeSet<String>,
    /// Filters the cloud and provides the label for a new tag.
    input: String,
}

impl TagSelection {
    /// Discard selected tags that are no longer available.
    fn retain_available(&mut self, tag_cloud: &TagCloud, edited_tags: &[String]) {
        if self.entity_uid.as_ref() != Some(&tag_cloud.entity_uid) {
            self.entity_uid = Some(tag_cloud.entity_uid.clone());
            self.track.clear();
        }
        self.track.retain(|label| edited_tags.contains(label));
        if !tag_cloud.is_loading() {
            self.cloud.retain(|label| tag_cloud.contains(label));
        }
    }
}

#[cfg(test)]
impl TagSelection {
    pub(super) fn selected_track_labels(&self) -> Vec<&str> {
        self.track.iter().map(String::as_str).collect()
    }
}

fn toggle(selected: &mut BTreeSet<String>, label: &str) {
    if !selected.remove(label) {
        selected.insert(label.to_owned());
    }
}

/// The labels when starting to drag a tag.
///
/// Dragging a selected tag drags all selected tags.
#[must_use]
fn dragged_labels(selected: &BTreeSet<String>, label: &str) -> Vec<String> {
    if selected.contains(label) {
        selected.iter().cloned().collect()
    } else {
        vec![label.to_owned()]
    }
}

/// Tags that are dragged from the cloud or from a track.
#[derive(Debug, Clone)]
pub(super) struct DraggedTags {
    /// The track that the tags are dragged from or `None` for the cloud.
    source: Option<TrackUid>,
    labels: Vec<String>,
}

/// Editing the tags while the tag cloud is open.
#[derive(Debug, Clone, Copy)]
pub(super) struct TagEditing<'a> {
    pub(super) tag_cloud: &'a TagCloud,
    pub(super) track_list: &'a [TrackListItem],
    pub(super) track_selection: Option<&'a TrackSelection>,
}

impl TagEditing<'_> {
    /// The tracks that are affected when editing the tags of a track.
    ///
    /// Editing a selected track affects all selected tracks.
    #[must_use]
    fn target_entity_uids<'a>(&'a self, entity_uid: &'a TrackUid) -> &'a [TrackUid] {
        match self.track_selection {
            Some(track_selection) if track_selection.contains(entity_uid) => {
                &track_selection.entity_uids
            }
            _ => std::slice::from_ref(entity_uid),
        }
    }

    /// Add and remove plain tags of the track and all other affected tracks.
    fn send_edit(
        &self,
        msg_tx: &MessageSender,
        entity_uid: &TrackUid,
        add: &[String],
        remove: &[String],
    ) {
        for entity_uid in self.target_entity_uids(entity_uid) {
            let tags = self
                .track_list
                .iter()
                .find(|item| item.entity_uid == *entity_uid)
                .map(|item| item.tags.as_slice());
            let Some(delta) = tags_delta(tags, add, remove) else {
                continue;
            };
            match MultiTrackEdit::new(MultiTrackFields::default(), delta) {
                Ok(edit) => msg_tx.send_action(TrackEditAction::Apply {
                    entity_uid: entity_uid.clone(),
                    edit: Box::new(edit),
                }),
                Err(errors) => log::error!("Invalid edit of track {entity_uid}: {errors:?}"),
            }
        }
    }
}

/// The modification of the current tags of a track.
///
/// Tracks with unknown tags are modified unconditionally.
///
/// Returns `None` if the tags would not change.
#[must_use]
fn tags_delta(
    tags: Option<&[String]>,
    add: &[String],
    remove: &[String],
) -> Option<CustomTagsDelta> {
    let contains = |label: &String| tags.map(|tags| tags.contains(label));
    let delta = CustomTagsDelta::plain(
        add.iter()
            .filter(|label| contains(label) != Some(true))
            .map(String::as_str),
        remove
            .iter()
            .filter(|label| contains(label) != Some(false))
            .map(String::as_str),
    );
    (!delta.is_empty()).then_some(delta)
}

/// Render the tags of a track as bubbles.
///
/// While the tag cloud is open the bubbles could be selected and dragged,
/// and tags could be dropped onto the cell.
///
/// Returns `true` if a bubble has been clicked without selecting it.
pub(super) fn render_tags_cell(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    item: &TrackListItem,
    tag_editing: Option<TagEditing<'_>>,
    selection: &mut TagSelection,
) -> bool {
    let Some(tag_editing) = tag_editing else {
        for tag in &item.tags {
            paint_bubble(ui, tag, Sense::hover(), false);
        }
        return false;
    };
    let cell_rect = ui.max_rect();
    let is_edited = tag_editing.tag_cloud.entity_uid == item.entity_uid;
    let mut is_clicked = false;
    for tag in &item.tags {
        let is_selected = is_edited && selection.track.contains(tag);
        let response = paint_bubble(ui, tag, Sense::click_and_drag(), is_selected);
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }
        if response.clicked() {
            if is_edited && ui.input(|input| input.modifiers.is_none()) {
                toggle(&mut selection.track, tag);
            } else {
                is_clicked = true;
            }
        }
        if response.drag_started() {
            let labels = if is_edited {
                dragged_labels(&selection.track, tag)
            } else {
                vec![tag.clone()]
            };
            response.dnd_set_drag_payload(DraggedTags {
                source: Some(item.entity_uid.clone()),
                labels,
            });
        }
    }
    let selection_color = ui.visuals().selection.bg_fill;
    if is_edited {
        ui.painter().rect_stroke(
            cell_rect,
            CornerRadius::ZERO,
            Stroke::new(EDITED_CELL_STROKE_WIDTH, selection_color),
            StrokeKind::Inside,
        );
    }
    if let Some(dragged) = DragAndDrop::payload::<DraggedTags>(ui.ctx())
        && dragged.source.as_ref() != Some(&item.entity_uid)
        && ui.rect_contains_pointer(cell_rect)
    {
        ui.painter().rect_stroke(
            cell_rect,
            CornerRadius::ZERO,
            Stroke::new(DROP_TARGET_STROKE_WIDTH, selection_color),
            StrokeKind::Inside,
        );
        if ui.input(|input| input.pointer.any_released()) {
            tag_editing.send_edit(msg_tx, &item.entity_uid, &dragged.labels, &[]);
            if dragged.source.is_none() {
                selection.cloud.clear();
            }
        }
    }
    is_clicked
}

/// Render the tag cloud in a separate window.
///
/// The window is a drop target for removing tags from tracks.
pub(super) fn render_tag_cloud_window(
    ctx: &Context,
    msg_tx: &MessageSender,
    tag_editing: TagEditing<'_>,
    is_editable: bool,
    selection: &mut TagSelection,
) {
    let tag_cloud = tag_editing.tag_cloud;
    let edited_item = tag_editing
        .track_list
        .iter()
        .find(|item| item.entity_uid == tag_cloud.entity_uid);
    let edited_tags = edited_item.map_or(&[][..], |item| item.tags.as_slice());
    selection.retain_available(tag_cloud, edited_tags);

    let mut is_open = true;
    let Some(window) = Window::new("Tags")
        .id(Id::unique(WINDOW_ID))
        .open(&mut is_open)
        .collapsible(false)
        .default_pos(WINDOW_DEFAULT_POS)
        .default_width(WINDOW_DEFAULT_WIDTH)
        .show(ctx, |ui| {
            render_header(ui, tag_editing, edited_item);
            ui.add_enabled_ui(is_editable, |ui| {
                render_input(ui, msg_tx, tag_editing, edited_tags, selection);
                render_selection_buttons(ui, msg_tx, tag_editing, selection);
                ui.separator();
                render_cloud(ui, msg_tx, tag_editing, edited_tags, selection);
            });
        })
    else {
        return;
    };
    if !is_open {
        msg_tx.send_action(TagCloudAction::Close);
        return;
    }
    if !is_editable {
        return;
    }
    if !selection.track.is_empty()
        && ctx.memory(|memory| memory.focused().is_none())
        && ctx.input(|input| input.key_pressed(Key::Delete))
    {
        let remove = std::mem::take(&mut selection.track)
            .into_iter()
            .collect::<Vec<_>>();
        tag_editing.send_edit(msg_tx, &tag_cloud.entity_uid, &[], &remove);
    }

    // Drop tags of a track into the window for removing them.
    let window_rect = window.response.rect;
    let layer_id = window.response.layer_id;
    if let Some(dragged) = DragAndDrop::payload::<DraggedTags>(ctx)
        && let Some(source) = &dragged.source
        && ctx.rect_contains_pointer(layer_id, window_rect)
    {
        ctx.layer_painter(layer_id).rect_stroke(
            window_rect,
            ctx.global_style().visuals.window_corner_radius,
            Stroke::new(
                DROP_TARGET_STROKE_WIDTH,
                ctx.global_style().visuals.selection.bg_fill,
            ),
            StrokeKind::Inside,
        );
        if ctx.input(|input| input.pointer.any_released()) {
            tag_editing.send_edit(msg_tx, source, &[], &dragged.labels);
        }
    }
}

fn render_header(ui: &mut Ui, tag_editing: TagEditing<'_>, edited_item: Option<&TrackListItem>) {
    ui.horizontal(|ui| {
        let track = edited_item.map_or_else(
            || "(not in the track list)".to_owned(),
            |item| {
                [item.artist.as_deref(), item.title.as_deref()]
                    .into_iter()
                    .flatten()
                    .join(" – ")
            },
        );
        ui.label("Track:");
        ui.strong(track);
        help_icon(
            ui,
            "Click tags to select them.\n\
             Drag tags from the cloud onto the tags of any track to add them.\n\
             Drag tags of a track into this window to remove them.\n\
             Double-click a tag in the cloud to add it to the edited track.\n\
             Type a label to filter the cloud or to create a new tag.\n\
             Press Delete to remove the selected tags of the edited track.\n\
             Edits of a selected track apply to all selected tracks.",
        );
    });
    let target_count = tag_editing
        .target_entity_uids(&tag_editing.tag_cloud.entity_uid)
        .len();
    if target_count > 1 {
        ui.label(format!(
            "Edits apply to all {target_count} selected tracks."
        ));
    }
}

/// Filter the cloud or create a new tag.
fn render_input(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    tag_editing: TagEditing<'_>,
    edited_tags: &[String],
    selection: &mut TagSelection,
) {
    let label = selection.input.trim().to_owned();
    let is_new = !label.is_empty() && !tag_editing.tag_cloud.contains(&label);
    let could_add = !label.is_empty() && !edited_tags.contains(&label);
    // Restricted to a single row. Otherwise the layout would occupy
    // all remaining space and hide the contents below.
    let row_size = Vec2::new(ui.available_width(), ui.spacing().interact_size.y);
    ui.allocate_ui_with_layout(row_size, Layout::right_to_left(Align::Center), |ui| {
        let (button_text, hover_text) = if is_new {
            ("Create", "Create a new tag and add it to the edited track.")
        } else {
            ("Add", "Add the tag to the edited track.")
        };
        let button = ui
            .add_enabled(could_add, Button::new(button_text))
            .on_hover_text(hover_text);
        let response = ui.add(
            TextEdit::singleline(&mut selection.input)
                .hint_text("Filter or create tag")
                .desired_width(f32::INFINITY),
        );
        let is_entered = response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));
        if could_add && (button.clicked() || is_entered) {
            tag_editing.send_edit(msg_tx, &tag_editing.tag_cloud.entity_uid, &[label], &[]);
            selection.input.clear();
            if is_entered {
                // Continue with the next tag.
                response.request_focus();
            }
        }
    });
}

fn render_selection_buttons(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    tag_editing: TagEditing<'_>,
    selection: &mut TagSelection,
) {
    let entity_uid = &tag_editing.tag_cloud.entity_uid;
    ui.horizontal(|ui| {
        let cloud_count = selection.cloud.len();
        if ui
            .add_enabled(
                cloud_count > 0,
                Button::new(format!("Add selected ({cloud_count})")),
            )
            .on_hover_text("Add the selected tags of the cloud to the edited track.")
            .clicked()
        {
            let add = std::mem::take(&mut selection.cloud)
                .into_iter()
                .collect::<Vec<_>>();
            tag_editing.send_edit(msg_tx, entity_uid, &add, &[]);
        }
        let track_count = selection.track.len();
        if ui
            .add_enabled(
                track_count > 0,
                Button::new(format!("Remove selected ({track_count})")),
            )
            .on_hover_text("Remove the selected tags from the edited track.")
            .clicked()
        {
            let remove = std::mem::take(&mut selection.track)
                .into_iter()
                .collect::<Vec<_>>();
            tag_editing.send_edit(msg_tx, entity_uid, &[], &remove);
        }
        if ui
            .add_enabled(
                cloud_count + track_count > 0,
                Button::new("Clear selection"),
            )
            .clicked()
        {
            selection.cloud.clear();
            selection.track.clear();
        }
    });
}

fn render_cloud(
    ui: &mut Ui,
    msg_tx: &MessageSender,
    tag_editing: TagEditing<'_>,
    edited_tags: &[String],
    selection: &mut TagSelection,
) {
    let tag_cloud = tag_editing.tag_cloud;
    if let Some(load_error) = &tag_cloud.load_error {
        ui.colored_label(ui.visuals().error_fg_color, load_error);
    }
    let label_counts = &tag_cloud.label_counts;
    if label_counts.is_empty() {
        if tag_cloud.is_loading() {
            ui.spinner();
        } else {
            ui.weak("No tags yet. Type a label above to create a new tag.");
        }
        return;
    }
    let filter = selection.input.trim().to_lowercase();
    let max_count = label_counts
        .iter()
        .map(|(_, count)| *count)
        .max()
        .unwrap_or_default();
    let body_font = TextStyle::Body.resolve(ui.style());
    ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (label, count) in label_counts.iter().filter(|(label, _)| {
                    filter.is_empty() || label.to_lowercase().contains(&filter)
                }) {
                    let is_assigned = edited_tags.contains(label);
                    let text_color = if is_assigned {
                        ui.visuals().weak_text_color()
                    } else {
                        ui.visuals().text_color()
                    };
                    let font_id = FontId::new(
                        body_font.size * font_scale(*count, max_count),
                        body_font.family.clone(),
                    );
                    let is_selected = selection.cloud.contains(label);
                    let tracks = if *count == 1 { "track" } else { "tracks" };
                    let hover_text = if is_assigned {
                        format!("{count} {tracks}, assigned to the edited track")
                    } else {
                        format!("{count} {tracks}")
                    };
                    let response = paint_bubble_with_font(
                        ui,
                        label,
                        font_id,
                        text_color,
                        Sense::click_and_drag(),
                        is_selected,
                    )
                    .on_hover_text(hover_text);
                    if response.hovered() {
                        ui.ctx().set_cursor_icon(CursorIcon::Grab);
                    }
                    if response.double_clicked() {
                        selection.cloud.remove(label);
                        tag_editing.send_edit(
                            msg_tx,
                            &tag_cloud.entity_uid,
                            std::slice::from_ref(label),
                            &[],
                        );
                    } else if response.clicked() {
                        toggle(&mut selection.cloud, label);
                    }
                    if response.drag_started() {
                        response.dnd_set_drag_payload(DraggedTags {
                            source: None,
                            labels: dragged_labels(&selection.cloud, label),
                        });
                    }
                }
            });
        });
}

/// Display the dragged tags next to the pointer.
pub(super) fn paint_dragged_tags(ctx: &Context) {
    let Some(dragged) = DragAndDrop::payload::<DraggedTags>(ctx) else {
        return;
    };
    let Some(pointer_pos) = ctx.pointer_interact_pos() else {
        return;
    };
    let mut text = dragged
        .labels
        .iter()
        .take(DRAGGED_TAGS_MAX_LABELS)
        .join(", ");
    if dragged.labels.len() > DRAGGED_TAGS_MAX_LABELS {
        text.push_str(", …");
    }
    let style = ctx.global_style();
    let painter = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::unique("dragged-tags")));
    let text_color = style.visuals.strong_text_color();
    let galley = painter.layout_no_wrap(text, TextStyle::Body.resolve(&style), text_color);
    let text_rect: Rect =
        Align2::LEFT_CENTER.anchor_size(pointer_pos + DRAGGED_TAGS_POINTER_OFFSET, galley.size());
    painter.rect_filled(
        text_rect.expand(DRAGGED_TAGS_MARGIN),
        style.visuals.menu_corner_radius,
        style.visuals.window_fill,
    );
    painter.galley(text_rect.min, galley, text_color);
}

#[cfg(test)]
mod tests;
