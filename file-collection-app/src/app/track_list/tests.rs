// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    cell::{Cell, RefCell},
    sync::mpsc,
};

use aoide::{EntityUid, TrackUid};
use egui::{
    CentralPanel, ColorImage, Context, Event, Id, Modifiers, PointerButton, Pos2, RawInput, Rect,
    TextureOptions, Vec2,
};

use crate::app::{Action, LibraryAction, Message};

use super::*;

#[test]
fn every_column_has_a_unique_id_that_can_be_parsed() {
    for column in TrackListColumn::ALL {
        assert_eq!(Some(column), TrackListColumn::from_id(column.id()));
    }
    let ids = TrackListColumn::ALL.map(TrackListColumn::id);
    assert_eq!(ids.len(), ids.iter().unique().count());
}

#[test]
fn newly_selected_columns_are_appended() {
    let mut columns = TrackListColumns {
        selected: vec![TrackListColumn::Key, TrackListColumn::Artwork],
    };
    assert!(columns.toggle(TrackListColumn::Title));
    assert_eq!(
        vec![
            TrackListColumn::Key,
            TrackListColumn::Artwork,
            TrackListColumn::Title
        ],
        columns.selected
    );
    assert!(columns.toggle(TrackListColumn::Artwork));
    assert_eq!(
        vec![TrackListColumn::Key, TrackListColumn::Title],
        columns.selected
    );
}

#[test]
fn move_column() {
    use TrackListColumn::{Artist as A, Bpm as B, Key as C, Title as D};
    let new_columns = || TrackListColumns {
        selected: vec![A, B, C, D],
    };
    let moved = |column, insertion_index| {
        let mut columns = new_columns();
        assert!(columns.move_column(column, insertion_index));
        columns.selected
    };
    assert_eq!(vec![B, C, A, D], moved(A, 3));
    assert_eq!(vec![B, C, D, A], moved(A, 4));
    assert_eq!(vec![D, A, B, C], moved(D, 0));
    assert_eq!(vec![A, C, B, D], moved(C, 1));
}

#[test]
fn moving_column_without_effect() {
    let mut columns = TrackListColumns {
        selected: vec![TrackListColumn::Artist, TrackListColumn::Title],
    };
    let unmodified = columns.clone();
    // Inserting before or after itself.
    assert!(!columns.move_column(TrackListColumn::Artist, 0));
    assert!(!columns.move_column(TrackListColumn::Artist, 1));
    assert!(!columns.move_column(TrackListColumn::Title, 2));
    // Out of bounds.
    assert!(!columns.move_column(TrackListColumn::Artist, 3));
    // Not selected.
    assert!(!columns.move_column(TrackListColumn::Key, 0));
    assert_eq!(unmodified, columns);
}

#[test]
fn last_selected_column_cannot_be_deselected() {
    let mut columns = TrackListColumns {
        selected: vec![TrackListColumn::Title],
    };
    assert!(!columns.toggle(TrackListColumn::Title));
    assert_eq!(vec![TrackListColumn::Title], columns.selected);
}

#[test]
fn json_roundtrip() {
    let columns = TrackListColumns {
        selected: vec![TrackListColumn::Title, TrackListColumn::FilePath],
    };
    assert_eq!(
        Some(columns.clone()),
        TrackListColumns::from_json(&columns.to_json())
    );
}

#[test]
fn from_json_sanitizes_ids() {
    assert_eq!(
        Some(TrackListColumns {
            selected: vec![TrackListColumn::Key, TrackListColumn::Title],
        }),
        TrackListColumns::from_json(br#"["key", "unknown", "title", "key"]"#)
    );
}

#[test]
fn from_json_rejects_contents_without_any_column() {
    assert_eq!(None, TrackListColumns::from_json(b"[]"));
    assert_eq!(None, TrackListColumns::from_json(br#"["unknown"]"#));
    assert_eq!(None, TrackListColumns::from_json(b"not json"));
}

fn new_item(ctx: &Context, title: &str) -> TrackListItem {
    TrackListItem {
        entity_uid: TrackUid::from_untyped(EntityUid::random()),
        content_url: None,
        is_synchronized: true,
        artist: None,
        title: Some(title.to_owned()),
        album_artist: None,
        album_title: None,
        album_subtitle: None,
        copyright: None,
        advisory_rating: None,
        grouping: None,
        comment: None,
        genres: Vec::new(),
        tags: Vec::new(),
        year: None,
        bpm: None,
        key: None,
        artwork_thumbnail_texture: ctx.load_texture(
            "",
            ColorImage::filled([4, 4], egui::Color32::BLACK),
            TextureOptions::LINEAR,
        ),
    }
}

struct Harness {
    ctx: Context,
    msg_tx: MessageSender,
    msg_rx: mpsc::Receiver<Message>,
    columns: TrackListColumns,
    table_state: RefCell<TrackListTableState>,
    track_list: Vec<TrackListItem>,
    is_draft_modified: Cell<bool>,
    tag_cloud: Option<TagCloud>,
}

impl Harness {
    fn new(columns: TrackListColumns, titles: &[&str]) -> Self {
        let ctx = Context::default();
        let (msg_tx, msg_rx) = mpsc::channel();
        let msg_tx = MessageSender::new(ctx.clone(), msg_tx);
        let track_list = titles.iter().map(|title| new_item(&ctx, title)).collect();
        Self {
            ctx,
            msg_tx,
            msg_rx,
            columns,
            table_state: RefCell::default(),
            track_list,
            is_draft_modified: Cell::new(false),
            tag_cloud: None,
        }
    }

    /// Render a single frame.
    fn frame(&self, mut events: Vec<Event>, modifiers: Modifiers, could_fetch_more: bool) {
        events.insert(0, Event::ModifiersChanged(modifiers));
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 600.0))),
            events,
            ..Default::default()
        };
        let mut output = self.ctx.run_ui(input, |ui| {
            CentralPanel::default().show(ui, |ui| {
                render_track_list(
                    ui,
                    &self.msg_tx,
                    &self.columns,
                    &mut self.table_state.borrow_mut(),
                    &self.track_list,
                    could_fetch_more,
                    TrackListInteraction {
                        edited_entity_uid: None,
                        track_selection: None,
                        is_track_editor_draft_modified: self.is_draft_modified.get(),
                        edit_error: None,
                        tag_cloud: self.tag_cloud.as_ref(),
                    },
                );
            });
        });
        // No renderer is available for applying the textures.
        output.textures_delta.clear();
    }

    fn click(&self, pos: Pos2, modifiers: Modifiers) {
        let pointer_event = |pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers,
        };
        self.frame(vec![Event::PointerMoved(pos)], modifiers, false);
        self.frame(vec![pointer_event(true)], modifiers, false);
        self.frame(vec![pointer_event(false)], modifiers, false);
        self.frame(Vec::new(), modifiers, false);
    }

    fn drag(&self, from: Pos2, to: Pos2) {
        let pointer_event = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        let frame = |event| self.frame(vec![event], Modifiers::NONE, false);
        frame(Event::PointerMoved(from));
        frame(pointer_event(from, true));
        // Exceed the drag threshold in a separate step.
        frame(Event::PointerMoved(from + Vec2::new(10.0, 0.0)));
        frame(Event::PointerMoved(to));
        frame(pointer_event(to, false));
        self.frame(Vec::new(), Modifiers::NONE, false);
    }

    fn actions(&self) -> Vec<LibraryAction> {
        self.msg_rx
            .try_iter()
            .map(|msg| {
                let Message::Action(Action::Library(action)) = msg else {
                    panic!("unexpected message: {msg:?}");
                };
                action
            })
            .collect()
    }
}

/// A position inside the title cell of the first row, right on top of the
/// text of the label.
const FIRST_ROW_LABEL_POS: Pos2 = Pos2::new(20.0, 62.0);

fn title_only_harness() -> Harness {
    Harness::new(
        TrackListColumns {
            selected: vec![TrackListColumn::Title],
        },
        &["First", "Second", "Third"],
    )
}

#[test]
fn clicking_on_a_non_text_cell_opens_the_track_editor() {
    let harness = Harness::new(
        TrackListColumns {
            selected: vec![TrackListColumn::Year],
        },
        &["First", "Second", "Third"],
    );
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    let actions = harness.actions();
    let [LibraryAction::TrackEdit(TrackEditAction::Open(entity_uid))] = actions.as_slice() else {
        panic!("unexpected actions: {actions:?}");
    };
    assert_eq!(harness.track_list[0].entity_uid, *entity_uid);
}

#[test]
fn clicking_on_a_row_with_modifiers_updates_the_selection() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::COMMAND);
    let actions = harness.actions();
    assert!(
        matches!(
            actions.as_slice(),
            [LibraryAction::TrackSelection(TrackSelectionAction::Toggle(
                _
            ))]
        ),
        "unexpected actions: {actions:?}"
    );
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::SHIFT);
    let actions = harness.actions();
    assert!(
        matches!(
            actions.as_slice(),
            [LibraryAction::TrackSelection(
                TrackSelectionAction::SelectRange(_)
            )]
        ),
        "unexpected actions: {actions:?}"
    );
}

#[test]
fn fetches_more_only_if_the_end_of_the_list_is_visible() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, true);
    let actions = harness.actions();
    assert!(
        matches!(
            actions.as_slice(),
            [LibraryAction::TrackSearch(TrackSearchAction::FetchMore)]
        ),
        "unexpected actions: {actions:?}"
    );

    let titles = vec!["Title"; 1000];
    let harness = Harness::new(TrackListColumns::default(), &titles);
    harness.frame(Vec::new(), Modifiers::NONE, true);
    assert!(harness.actions().is_empty());
}

#[test]
fn renders_all_columns_without_panicking() {
    let harness = Harness::new(
        TrackListColumns {
            selected: TrackListColumn::ALL.to_vec(),
        },
        &["First", "Second"],
    );
    // The initial sizing pass is followed by regular frames.
    for _ in 0..3 {
        harness.frame(Vec::new(), Modifiers::NONE, false);
    }
}

#[test]
fn switching_columns_does_not_panic() {
    let mut harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.columns = TrackListColumns::default();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.columns.toggle(TrackListColumn::Year);
    harness.frame(Vec::new(), Modifiers::NONE, false);
}

#[test]
fn columns_menu_stays_open_for_toggling_multiple_columns() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    // Open the menu that is located in the top left corner.
    harness.click(Pos2::new(24.0, 16.0), Modifiers::NONE);
    assert!(harness.actions().is_empty());
    // Click repeatedly while moving downwards along the entries of the menu.
    let mut toggled = Vec::new();
    for y in (24..200).step_by(4) {
        harness.click(Pos2::new(30.0, y as f32), Modifiers::NONE);
        for action in harness.msg_rx.try_iter() {
            let Message::Action(Action::TrackListColumns(TrackListColumnsAction::Toggle(column))) =
                action
            else {
                panic!("unexpected message: {action:?}");
            };
            toggled.push(column);
        }
    }
    assert!(toggled.contains(&TrackListColumn::Artwork), "{toggled:?}");
    assert!(
        toggled.contains(&TrackListColumn::Unsynchronized),
        "{toggled:?}"
    );
    // The only selected column cannot be deselected.
    assert!(!toggled.contains(&TrackListColumn::Title), "{toggled:?}");
}

#[test]
fn saved_columns_are_restored() {
    let settings_dir = std::env::temp_dir().join(format!(
        "aoide_track_list_columns_test_{pid}",
        pid = std::process::id()
    ));
    fs::create_dir_all(&settings_dir).unwrap();
    assert_eq!(
        TrackListColumns::default(),
        TrackListColumns::restore(&settings_dir)
    );
    let mut columns = TrackListColumns::default();
    assert!(columns.toggle(TrackListColumn::Year));
    assert!(columns.toggle(TrackListColumn::Artwork));
    assert!(columns.move_column(TrackListColumn::Year, 0));
    columns.save(&settings_dir).unwrap();
    assert_eq!(columns, TrackListColumns::restore(&settings_dir));
    // Invalid contents are replaced by the defaults.
    fs::write(TrackListColumns::file_path(&settings_dir), b"invalid").unwrap();
    assert_eq!(
        TrackListColumns::default(),
        TrackListColumns::restore(&settings_dir)
    );
    fs::remove_dir_all(&settings_dir).unwrap();
}

#[test]
fn rows_remain_clickable_if_the_columns_exceed_the_available_width() {
    let harness = Harness::new(
        TrackListColumns {
            selected: TrackListColumn::ALL.to_vec(),
        },
        &["First"],
    );
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(Pos2::new(900.0, FIRST_ROW_LABEL_POS.y), Modifiers::NONE);
    // A text column.
    assert!(harness.is_editing());
}

/// The vertical center of the table header.
const HEADER_Y: f32 = 44.0;

/// Cells at x = [8..168], [176..416], and [424..].
fn three_columns_harness() -> Harness {
    let harness = Harness::new(
        TrackListColumns {
            selected: vec![
                TrackListColumn::Artist,
                TrackListColumn::Title,
                TrackListColumn::Key,
            ],
        },
        &["First"],
    );
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness
}

fn drag_slowly(harness: &Harness, from: Pos2, to: Pos2) {
    let pointer_event = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    let frame = |event| harness.frame(vec![event], Modifiers::NONE, false);
    frame(Event::PointerMoved(from));
    frame(pointer_event(from, true));
    for step in 1..=20 {
        frame(Event::PointerMoved(
            from + (to - from) * (step as f32 / 20.0),
        ));
    }
    frame(pointer_event(to, false));
    harness.frame(Vec::new(), Modifiers::NONE, false);
}

fn move_actions(harness: &Harness) -> Vec<(TrackListColumn, usize)> {
    harness
        .msg_rx
        .try_iter()
        .filter_map(|msg| match msg {
            Message::Action(Action::TrackListColumns(TrackListColumnsAction::Move {
                column,
                insertion_index,
            })) => Some((column, insertion_index)),
            _ => None,
        })
        .collect()
}

#[test]
fn column_insertion_index_depends_on_the_center_of_the_headers() {
    let rects = [
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 10.0)),
        Rect::from_min_max(Pos2::new(100.0, 0.0), Pos2::new(200.0, 10.0)),
    ];
    assert_eq!(0, column_insertion_index(&rects, -10.0));
    assert_eq!(0, column_insertion_index(&rects, 49.0));
    assert_eq!(1, column_insertion_index(&rects, 51.0));
    assert_eq!(1, column_insertion_index(&rects, 149.0));
    assert_eq!(2, column_insertion_index(&rects, 151.0));
    assert_eq!(2, column_insertion_index(&rects, 2000.0));
}

#[test]
fn dragging_a_header_moves_the_column() {
    let harness = three_columns_harness();
    // Drop onto the right half of the title column.
    harness.drag(Pos2::new(50.0, HEADER_Y), Pos2::new(380.0, HEADER_Y));
    assert_eq!(vec![(TrackListColumn::Artist, 2)], move_actions(&harness));
}

#[test]
fn columns_could_be_dropped_at_any_vertical_position() {
    for drop_y in [4.0, 20.0, HEADER_Y, 60.0, 120.0, 400.0, 590.0] {
        let harness = three_columns_harness();
        // The grabbed position could be on top of the text.
        drag_slowly(
            &harness,
            Pos2::new(16.0, HEADER_Y),
            Pos2::new(380.0, drop_y),
        );
        assert_eq!(
            vec![(TrackListColumn::Artist, 2)],
            move_actions(&harness),
            "dropped at y = {drop_y}"
        );
    }
}

#[test]
fn dropping_beyond_the_headers_moves_the_column_to_the_end_or_start() {
    let harness = three_columns_harness();
    drag_slowly(&harness, Pos2::new(50.0, HEADER_Y), Pos2::new(995.0, 300.0));
    assert_eq!(vec![(TrackListColumn::Artist, 3)], move_actions(&harness));

    let harness = three_columns_harness();
    drag_slowly(&harness, Pos2::new(500.0, HEADER_Y), Pos2::new(1.0, 300.0));
    assert_eq!(vec![(TrackListColumn::Key, 0)], move_actions(&harness));
}

#[test]
fn consecutive_moves_are_applied() {
    let mut harness = three_columns_harness();
    for (from_x, to_x, expected) in [
        (
            50.0,
            380.0,
            [
                TrackListColumn::Title,
                TrackListColumn::Artist,
                TrackListColumn::Key,
            ],
        ),
        (
            500.0,
            20.0,
            [
                TrackListColumn::Key,
                TrackListColumn::Title,
                TrackListColumn::Artist,
            ],
        ),
    ] {
        drag_slowly(
            &harness,
            Pos2::new(from_x, HEADER_Y),
            Pos2::new(to_x, 200.0),
        );
        let actions = move_actions(&harness);
        assert_eq!(1, actions.len(), "{actions:?}");
        for (column, insertion_index) in actions {
            assert!(harness.columns.move_column(column, insertion_index));
        }
        assert_eq!(expected.as_slice(), harness.columns.selected.as_slice());
        harness.frame(Vec::new(), Modifiers::NONE, false);
    }
}

#[test]
fn dragging_a_row_does_not_move_columns() {
    let harness = three_columns_harness();
    drag_slowly(
        &harness,
        FIRST_ROW_LABEL_POS,
        Pos2::new(380.0, FIRST_ROW_LABEL_POS.y),
    );
    assert_eq!(0, harness.msg_rx.try_iter().count());
}

#[test]
fn dragging_a_header_onto_itself_has_no_effect() {
    let harness = three_columns_harness();
    harness.drag(Pos2::new(50.0, HEADER_Y), Pos2::new(120.0, HEADER_Y));
    assert_eq!(0, harness.msg_rx.try_iter().count());
}

#[test]
fn dragging_the_border_of_a_header_resizes_the_column() {
    let harness = three_columns_harness();
    harness.drag(Pos2::new(172.0, HEADER_Y), Pos2::new(212.0, HEADER_Y));
    assert_eq!(0, harness.msg_rx.try_iter().count());
    let width = harness.table_state.borrow().widths[&TrackListColumn::Artist];
    assert!(width > 180.0, "{width}");
}

#[test]
fn moving_columns_preserves_their_widths() {
    let mut harness = three_columns_harness();
    harness.drag(Pos2::new(172.0, HEADER_Y), Pos2::new(212.0, HEADER_Y));
    let width_before = harness.table_state.borrow().widths[&TrackListColumn::Artist];
    assert!(harness.columns.move_column(TrackListColumn::Artist, 2));
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.frame(Vec::new(), Modifiers::NONE, false);
    let width_after = harness.table_state.borrow().widths[&TrackListColumn::Artist];
    assert!(
        (width_before - width_after).abs() < 1.0,
        "{width_before} {width_after}"
    );
}

fn key_event(key: egui::Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

impl Harness {
    fn press_key(&self, key: egui::Key) {
        self.frame(vec![key_event(key)], Modifiers::NONE, false);
        self.frame(Vec::new(), Modifiers::NONE, false);
    }

    fn type_text(&self, text: &str) {
        self.frame(vec![Event::Text(text.to_owned())], Modifiers::NONE, false);
        self.frame(Vec::new(), Modifiers::NONE, false);
    }

    fn is_editing(&self) -> bool {
        self.table_state.borrow().cell_edit.is_some()
    }
}

fn expected_edit(fields: MultiTrackFields) -> MultiTrackEdit {
    MultiTrackEdit::new(fields, CustomTagsDelta::default()).unwrap()
}

fn assert_single_edit(harness: &Harness, index: usize, fields: MultiTrackFields) {
    let actions = harness.actions();
    let [LibraryAction::TrackEdit(TrackEditAction::Apply { entity_uid, edit })] =
        actions.as_slice()
    else {
        panic!("unexpected actions: {actions:?}");
    };
    assert_eq!(harness.track_list[index].entity_uid, *entity_uid);
    assert_eq!(expected_edit(fields), **edit);
}

fn title_fields(title: &str) -> MultiTrackFields {
    MultiTrackFields {
        title: Some(title.to_owned()),
        ..Default::default()
    }
}

#[test]
fn clicking_into_a_text_cell_starts_editing_and_enter_applies() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    assert!(harness.is_editing());
    assert!(harness.msg_rx.try_iter().next().is_none());
    harness.type_text("!");
    harness.press_key(egui::Key::Enter);
    assert!(!harness.is_editing());
    assert_single_edit(&harness, 0, title_fields("First!"));
}

#[test]
fn escape_aborts_editing() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    harness.type_text("!");
    harness.press_key(egui::Key::Escape);
    assert!(!harness.is_editing());
    assert_eq!(0, harness.msg_rx.try_iter().count());
    // The text is not resurrected when editing the cell again.
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    harness.press_key(egui::Key::Enter);
    assert_eq!(0, harness.msg_rx.try_iter().count());
}

#[test]
fn unmodified_text_is_not_applied() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    harness.press_key(egui::Key::Enter);
    assert!(!harness.is_editing());
    assert_eq!(0, harness.msg_rx.try_iter().count());
}

#[test]
fn clicking_elsewhere_applies_the_text() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    harness.type_text("!");
    // The empty space below the table.
    harness.click(Pos2::new(300.0, 400.0), Modifiers::NONE);
    assert!(!harness.is_editing());
    assert_single_edit(&harness, 0, title_fields("First!"));
}

#[test]
fn clicking_into_another_cell_applies_the_text_and_edits_that_cell() {
    let row_height = 19.0;
    for (from_y, to_y, from_index, to_index) in [
        (
            FIRST_ROW_LABEL_POS.y,
            FIRST_ROW_LABEL_POS.y + row_height,
            0,
            1,
        ),
        // The target row is rendered before the edited row.
        (
            FIRST_ROW_LABEL_POS.y + row_height,
            FIRST_ROW_LABEL_POS.y,
            1,
            0,
        ),
    ] {
        let harness = title_only_harness();
        harness.frame(Vec::new(), Modifiers::NONE, false);
        harness.click(Pos2::new(20.0, from_y), Modifiers::NONE);
        harness.type_text("!");
        harness.click(Pos2::new(20.0, to_y), Modifiers::NONE);
        let expected_title = format!(
            "{title}!",
            title = harness.track_list[from_index].title.as_deref().unwrap()
        );
        assert_single_edit(&harness, from_index, title_fields(&expected_title));
        let table_state = harness.table_state.borrow();
        let cell_edit = table_state.cell_edit.as_ref().unwrap();
        assert_eq!(
            harness.track_list[to_index].entity_uid,
            cell_edit.entity_uid
        );
    }
}

#[test]
fn text_cells_are_not_editable_with_a_modified_track_editor_draft() {
    let harness = title_only_harness();
    harness.is_draft_modified.set(true);
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    assert!(!harness.is_editing());
    assert_eq!(0, harness.msg_rx.try_iter().count());
}

#[test]
fn clicking_a_text_cell_with_modifiers_selects_instead_of_editing() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::COMMAND);
    assert!(!harness.is_editing());
}

#[test]
fn multiple_values_open_the_track_editor_instead_of_editing_the_cell() {
    let mut harness = Harness::new(
        TrackListColumns {
            selected: vec![TrackListColumn::Comment],
        },
        &["First"],
    );
    harness.track_list[0].comment = Some("One\nTwo".to_owned());
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    assert!(!harness.is_editing());
    let actions = harness.actions();
    assert!(
        matches!(
            actions.as_slice(),
            [LibraryAction::TrackEdit(TrackEditAction::Open(_))]
        ),
        "unexpected actions: {actions:?}"
    );
}

#[test]
fn right_clicking_a_row_offers_to_open_the_track_editor() {
    let harness = title_only_harness();
    harness.frame(Vec::new(), Modifiers::NONE, false);
    let pos = FIRST_ROW_LABEL_POS;
    let pointer_event = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Secondary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    harness.frame(vec![Event::PointerMoved(pos)], Modifiers::NONE, false);
    harness.frame(vec![pointer_event(true)], Modifiers::NONE, false);
    harness.frame(vec![pointer_event(false)], Modifiers::NONE, false);
    harness.frame(Vec::new(), Modifiers::NONE, false);
    assert!(!harness.is_editing());
    // The button is located right below the pointer.
    harness.click(pos + Vec2::new(16.0, 16.0), Modifiers::NONE);
    let actions = harness.actions();
    assert!(
        matches!(
            actions.as_slice(),
            [LibraryAction::TrackEdit(TrackEditAction::Open(_))]
        ),
        "unexpected actions: {actions:?}"
    );
}

fn genres_harness() -> Harness {
    let mut harness = Harness::new(
        TrackListColumns {
            selected: vec![TrackListColumn::Genres],
        },
        &["First"],
    );
    harness.track_list[0].genres = vec!["Ambient".to_owned(), "Techno".to_owned()];
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness
}

fn genres_fields(genres: &str) -> MultiTrackFields {
    MultiTrackFields {
        genres: Some(genres.to_owned()),
        ..Default::default()
    }
}

#[test]
fn dragging_a_genre_reorders_the_genres() {
    let harness = genres_harness();
    drag_slowly(&harness, FIRST_ROW_LABEL_POS, Pos2::new(300.0, 62.0));
    assert_single_edit(&harness, 0, genres_fields("Techno\nAmbient"));
}

#[test]
fn dragging_a_genre_onto_its_position_has_no_effect() {
    let harness = genres_harness();
    drag_slowly(&harness, FIRST_ROW_LABEL_POS, Pos2::new(30.0, 62.0));
    assert_eq!(0, harness.msg_rx.try_iter().count());
}

#[test]
fn genres_are_not_movable_with_a_modified_track_editor_draft() {
    let harness = genres_harness();
    harness.is_draft_modified.set(true);
    harness.frame(Vec::new(), Modifiers::NONE, false);
    drag_slowly(&harness, FIRST_ROW_LABEL_POS, Pos2::new(300.0, 62.0));
    assert_eq!(0, harness.msg_rx.try_iter().count());
}

#[test]
fn clicking_a_genre_opens_the_track_editor() {
    let harness = genres_harness();
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    let actions = harness.actions();
    assert!(
        matches!(
            actions.as_slice(),
            [LibraryAction::TrackEdit(TrackEditAction::Open(_))]
        ),
        "unexpected actions: {actions:?}"
    );
}

#[test]
fn tags_are_not_editable_as_text() {
    let ctx = Context::default();
    let mut item = new_item(&ctx, "Title");
    item.tags = vec!["chill".to_owned(), "vocal".to_owned()];
    assert_eq!(
        Some("chill, vocal".to_owned()),
        TrackListColumn::Tags.cell_text(&item)
    );
    assert_eq!(None, TrackListColumn::Tags.editable_text(&item));
}

/// Two tracks with tags in the first row.
fn tags_harness(is_tag_cloud_open: bool) -> Harness {
    let mut harness = Harness::new(
        TrackListColumns {
            selected: vec![TrackListColumn::Tags],
        },
        &["First", "Second"],
    );
    harness.track_list[0].tags = vec!["chill".to_owned(), "vocal".to_owned()];
    if is_tag_cloud_open {
        harness.tag_cloud = Some(TagCloud::new(harness.track_list[0].entity_uid.clone()));
    }
    harness.frame(Vec::new(), Modifiers::NONE, false);
    harness
}

const ROW_HEIGHT: f32 = 19.0;

/// An empty position in the tags cell of the second row.
const SECOND_ROW_EMPTY_POS: Pos2 = Pos2::new(300.0, FIRST_ROW_LABEL_POS.y + ROW_HEIGHT);

impl Harness {
    fn all_actions(&self) -> Vec<Action> {
        self.msg_rx
            .try_iter()
            .map(|msg| {
                let Message::Action(action) = msg else {
                    panic!("unexpected message: {msg:?}");
                };
                action
            })
            .collect()
    }

    fn tag_cloud_window_rect(&self) -> Rect {
        self.ctx
            .memory(|memory| memory.area_rect(Id::unique("tag-cloud")))
            .expect("tag cloud window")
    }
}

fn assert_single_tags_edit(harness: &Harness, index: usize, add: &[&str], remove: &[&str]) {
    let expected_edit = MultiTrackEdit::new(
        MultiTrackFields::default(),
        CustomTagsDelta::plain(add.iter().copied(), remove.iter().copied()),
    )
    .unwrap();
    let actions = harness.actions();
    let [LibraryAction::TrackEdit(TrackEditAction::Apply { entity_uid, edit })] =
        actions.as_slice()
    else {
        panic!("unexpected actions: {actions:?}");
    };
    assert_eq!(harness.track_list[index].entity_uid, *entity_uid);
    assert_eq!(expected_edit, **edit);
}

#[test]
fn clicking_tags_opens_the_tag_cloud() {
    for pos in [FIRST_ROW_LABEL_POS, Pos2::new(300.0, FIRST_ROW_LABEL_POS.y)] {
        let harness = tags_harness(false);
        harness.click(pos, Modifiers::NONE);
        let actions = harness.all_actions();
        let [Action::TagCloud(TagCloudAction::Open(entity_uid))] = actions.as_slice() else {
            panic!("unexpected actions: {actions:?}");
        };
        assert_eq!(harness.track_list[0].entity_uid, *entity_uid);
    }
}

#[test]
fn clicking_tags_with_modifiers_selects_the_track() {
    let harness = tags_harness(false);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::COMMAND);
    let actions = harness.actions();
    assert!(
        matches!(
            actions.as_slice(),
            [LibraryAction::TrackSelection(TrackSelectionAction::Toggle(
                _
            ))]
        ),
        "unexpected actions: {actions:?}"
    );
}

#[test]
fn clicking_tags_of_another_track_switches_the_edited_track() {
    let harness = tags_harness(true);
    harness.click(SECOND_ROW_EMPTY_POS, Modifiers::NONE);
    let actions = harness.all_actions();
    let [Action::TagCloud(TagCloudAction::Open(entity_uid))] = actions.as_slice() else {
        panic!("unexpected actions: {actions:?}");
    };
    assert_eq!(harness.track_list[1].entity_uid, *entity_uid);
}

#[test]
fn clicking_a_tag_of_the_edited_track_selects_it() {
    let harness = tags_harness(true);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    assert_eq!(0, harness.msg_rx.try_iter().count());
    assert_eq!(
        vec!["chill"],
        harness
            .table_state
            .borrow()
            .tag_selection
            .selected_track_labels()
    );
    // Toggled
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    assert_eq!(
        Vec::<&str>::new(),
        harness
            .table_state
            .borrow()
            .tag_selection
            .selected_track_labels()
    );
}

#[test]
fn dragging_a_tag_onto_another_track_adds_it() {
    let harness = tags_harness(true);
    drag_slowly(&harness, FIRST_ROW_LABEL_POS, SECOND_ROW_EMPTY_POS);
    assert_single_tags_edit(&harness, 1, &["chill"], &[]);
}

#[test]
fn dragging_selected_tags_of_the_edited_track_drags_all_of_them() {
    let harness = tags_harness(true);
    let vocal_pos = Pos2::new(FIRST_ROW_LABEL_POS.x + 50.0, FIRST_ROW_LABEL_POS.y);
    harness.click(FIRST_ROW_LABEL_POS, Modifiers::NONE);
    harness.click(vocal_pos, Modifiers::NONE);
    assert_eq!(
        vec!["chill", "vocal"],
        harness
            .table_state
            .borrow()
            .tag_selection
            .selected_track_labels()
    );
    drag_slowly(&harness, vocal_pos, SECOND_ROW_EMPTY_POS);
    assert_single_tags_edit(&harness, 1, &["chill", "vocal"], &[]);
}

#[test]
fn dragging_a_tag_onto_its_own_track_has_no_effect() {
    let harness = tags_harness(true);
    drag_slowly(
        &harness,
        FIRST_ROW_LABEL_POS,
        Pos2::new(300.0, FIRST_ROW_LABEL_POS.y),
    );
    assert_eq!(0, harness.msg_rx.try_iter().count());
}

#[test]
fn dragging_a_tag_into_the_tag_cloud_removes_it() {
    let harness = tags_harness(true);
    let window_rect = harness.tag_cloud_window_rect();
    drag_slowly(&harness, FIRST_ROW_LABEL_POS, window_rect.center());
    assert_single_tags_edit(&harness, 0, &[], &["chill"]);
}

#[test]
fn tags_are_not_editable_with_a_modified_track_editor_draft() {
    let harness = tags_harness(true);
    harness.is_draft_modified.set(true);
    harness.frame(Vec::new(), Modifiers::NONE, false);
    drag_slowly(&harness, FIRST_ROW_LABEL_POS, SECOND_ROW_EMPTY_POS);
    let window_rect = harness.tag_cloud_window_rect();
    drag_slowly(&harness, FIRST_ROW_LABEL_POS, window_rect.center());
    harness.click(SECOND_ROW_EMPTY_POS, Modifiers::NONE);
    assert_eq!(0, harness.msg_rx.try_iter().count());
}

#[test]
fn tag_cloud_window_fits_its_contents() {
    let mut harness = tags_harness(true);
    let tag_cloud = harness.tag_cloud.as_mut().unwrap();
    let request_id = tag_cloud.start_loading();
    assert!(tag_cloud.finish_loading(
        request_id,
        Ok(vec![("chill".to_owned(), 2), ("deep".to_owned(), 1)])
    ));
    for _ in 0..3 {
        harness.frame(Vec::new(), Modifiers::NONE, false);
    }
    let window_rect = harness.tag_cloud_window_rect();
    // Header, input, buttons, separator, and a single line of tags.
    assert!(window_rect.height() < 200.0, "{window_rect:?}");
}
