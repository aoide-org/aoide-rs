// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::mpsc;

use egui::{
    Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2, ViewportEvent,
    ViewportId, ViewportInfo,
};

use aoide::{
    media::{
        self,
        content::{AudioContentMetadata, ContentLink, ContentRevision},
    },
    util::clock::OffsetDateTimeMs,
};

use super::super::message::{Action, Message};
use super::*;

fn no_pending_edits() -> PendingEdits {
    PendingEdits {
        draft: DraftEdit::Unmodified,
        modified_track_count: Some(0),
        is_counting: false,
        is_writing: false,
        could_write_files: true,
    }
}

fn modified_tracks(count: usize) -> PendingEdits {
    PendingEdits {
        modified_track_count: Some(count),
        ..no_pending_edits()
    }
}

fn new_track() -> Box<Track> {
    Box::new(Track::new_from_media_source(media::Source {
        collected_at: OffsetDateTimeMs::now_utc(),
        content: media::Content {
            link: ContentLink {
                path: "track.mp3".into(),
                rev: ContentRevision::new(1).into(),
            },
            r#type: "audio/mpeg".parse().unwrap(),
            metadata_flags: Default::default(),
            metadata: AudioContentMetadata::default().into(),
            digest: None,
        },
        artwork: Default::default(),
    }))
}

struct Harness {
    ctx: Context,
    msg_tx: MessageSender,
    msg_rx: mpsc::Receiver<Message>,
}

impl Harness {
    fn new() -> Self {
        let ctx = Context::default();
        let (msg_tx, msg_rx) = mpsc::channel();
        let msg_tx = MessageSender::new(ctx.clone(), msg_tx);
        Self {
            ctx,
            msg_tx,
            msg_rx,
        }
    }

    fn raw_input(events: Vec<Event>, close_requested: bool) -> RawInput {
        let mut raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 800.0))),
            events,
            ..Default::default()
        };
        raw_input.viewports.insert(
            ViewportId::ROOT,
            ViewportInfo {
                events: if close_requested {
                    vec![ViewportEvent::Close]
                } else {
                    Vec::new()
                },
                ..Default::default()
            },
        );
        raw_input
    }

    /// Run a frame and return the commands for the root viewport.
    fn run_close_frame(
        &self,
        close_requested: bool,
        close_state: &mut CloseState,
        pending_edits: &PendingEdits,
        finish_cell_edit: impl FnOnce(),
    ) -> Vec<ViewportCommand> {
        let mut finish_cell_edit = Some(finish_cell_edit);
        let mut output = self
            .ctx
            .run_ui(Self::raw_input(Vec::new(), close_requested), |ui| {
                *close_state = handle_close(
                    ui.ctx(),
                    &self.msg_tx,
                    *close_state,
                    pending_edits,
                    finish_cell_edit.take().unwrap(),
                );
            });
        output.textures_delta.clear();
        output
            .viewport_output
            .remove(&ViewportId::ROOT)
            .map_or_default(|viewport| viewport.commands)
    }

    /// Run a frame and return the area of the dialog if it has been rendered.
    fn run_dialog_frame(&self, events: Vec<Event>, pending_edits: &PendingEdits) -> Option<Rect> {
        let mut dialog_rect = None;
        let mut output = self.ctx.run_ui(Self::raw_input(events, false), |ui| {
            render_close_dialog(ui.ctx(), &self.msg_tx, pending_edits, None);
            dialog_rect = ui.ctx().memory(|mem| mem.area_rect(close_dialog_id()));
        });
        output.textures_delta.clear();
        dialog_rect
    }

    fn actions(&self) -> Vec<PendingEditsAction> {
        self.msg_rx
            .try_iter()
            .filter_map(|msg| match msg {
                Message::Action(Action::PendingEdits(action)) => Some(action),
                _ => None,
            })
            .collect()
    }

    fn press_escape(&self, pending_edits: &PendingEdits) {
        self.run_dialog_frame(Vec::new(), pending_edits);
        self.run_dialog_frame(
            vec![Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
            pending_edits,
        );
    }
}

fn click_events(pos: Pos2) -> Vec<Vec<Event>> {
    let pointer_button = |pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    vec![
        vec![Event::PointerMoved(pos)],
        vec![pointer_button(true)],
        vec![pointer_button(false)],
    ]
}

#[test]
fn only_the_result_of_the_latest_count_request_is_accepted() {
    let mut count = ModifiedTrackCount::default();
    let outdated = count.start_counting();
    let latest = count.start_counting();
    assert!(!count.finish_counting(outdated, Some(1)));
    assert!(count.is_counting);
    assert_eq!(None, count.count);
    assert!(count.finish_counting(latest, Some(2)));
    assert!(!count.is_counting);
    assert_eq!(Some(2), count.count);
}

#[test]
fn reset_rejects_pending_count_requests() {
    let mut count = ModifiedTrackCount::default();
    let pending = count.start_counting();
    count.reset();
    assert!(!count.finish_counting(pending, Some(1)));
    assert!(!count.is_counting);
    assert_eq!(Some(0), count.count);
}

#[test]
fn nothing_is_pending_only_if_everything_is_known_and_finished() {
    assert!(no_pending_edits().is_empty());
    assert!(!modified_tracks(1).is_empty());
    for pending_edits in [
        PendingEdits {
            draft: DraftEdit::Valid(new_track()),
            ..no_pending_edits()
        },
        PendingEdits {
            draft: DraftEdit::Invalid,
            ..no_pending_edits()
        },
        PendingEdits {
            modified_track_count: None,
            ..no_pending_edits()
        },
        PendingEdits {
            is_counting: true,
            ..no_pending_edits()
        },
        PendingEdits {
            is_writing: true,
            ..no_pending_edits()
        },
    ] {
        assert!(!pending_edits.is_empty(), "{pending_edits:?}");
    }
}

#[test]
fn saving_requires_pending_edits_and_an_idle_library() {
    assert!(!no_pending_edits().could_save());
    assert!(modified_tracks(1).could_save());
    assert!(
        PendingEdits {
            draft: DraftEdit::Valid(new_track()),
            ..no_pending_edits()
        }
        .could_save()
    );
    assert!(
        PendingEdits {
            modified_track_count: None,
            ..no_pending_edits()
        }
        .could_save()
    );
    assert!(
        !PendingEdits {
            is_writing: true,
            ..modified_tracks(1)
        }
        .could_save()
    );
    assert!(
        !PendingEdits {
            could_write_files: false,
            ..modified_tracks(1)
        }
        .could_save()
    );
    // Writing the files would succeed, but the draft would get lost.
    assert!(
        !PendingEdits {
            draft: DraftEdit::Invalid,
            ..modified_tracks(1)
        }
        .could_save()
    );
}

#[test]
fn the_save_action_contains_only_a_valid_draft() {
    assert!(matches!(
        modified_tracks(1).save_action(),
        PendingEditsAction::Save { draft: None }
    ));
    assert!(matches!(
        PendingEdits {
            draft: DraftEdit::Valid(new_track()),
            ..no_pending_edits()
        }
        .save_action(),
        PendingEditsAction::Save { draft: Some(_) }
    ));
}

#[test]
fn the_save_button_shows_the_number_of_modified_tracks() {
    assert_eq!("Save pending edits", no_pending_edits().save_button_text());
    assert_eq!(
        "Save pending edits (3)",
        modified_tracks(3).save_button_text()
    );
}

#[test]
fn close_request_is_cancelled_and_finishes_the_cell_edit() {
    let harness = Harness::new();
    let mut close_state = CloseState::NotRequested;
    let mut is_cell_edit_finished = false;
    let commands = harness.run_close_frame(true, &mut close_state, &no_pending_edits(), || {
        is_cell_edit_finished = true;
    });
    assert!(commands.contains(&ViewportCommand::CancelClose));
    assert!(is_cell_edit_finished);
    assert!(matches!(
        harness.actions().as_slice(),
        [PendingEditsAction::RequestClose]
    ));
}

#[test]
fn closes_after_nothing_is_pending_anymore() {
    let harness = Harness::new();
    let mut close_state = CloseState::Requested;

    let commands = harness.run_close_frame(false, &mut close_state, &modified_tracks(1), || {});
    assert!(!commands.contains(&ViewportCommand::Close));
    assert_eq!(CloseState::Requested, close_state);

    let commands = harness.run_close_frame(false, &mut close_state, &no_pending_edits(), || {});
    assert!(commands.contains(&ViewportCommand::Close));
    assert_eq!(CloseState::Confirmed, close_state);

    // The resulting close request must pass.
    let commands = harness.run_close_frame(true, &mut close_state, &no_pending_edits(), || {});
    assert!(!commands.contains(&ViewportCommand::CancelClose));
}

#[test]
fn does_not_close_without_a_request() {
    let harness = Harness::new();
    let mut close_state = CloseState::NotRequested;
    let commands = harness.run_close_frame(false, &mut close_state, &no_pending_edits(), || {});
    assert!(!commands.contains(&ViewportCommand::Close));
    assert_eq!(CloseState::NotRequested, close_state);
}

#[test]
fn repeated_close_requests_keep_the_dialog_open() {
    let harness = Harness::new();
    let mut close_state = CloseState::Requested;
    let commands = harness.run_close_frame(true, &mut close_state, &modified_tracks(1), || {
        panic!("not editing a cell anymore");
    });
    assert!(commands.contains(&ViewportCommand::CancelClose));
    assert_eq!(CloseState::Requested, close_state);
    assert!(harness.actions().is_empty());
}

#[test]
fn escape_cancels_closing() {
    let harness = Harness::new();
    harness.press_escape(&modified_tracks(1));
    assert!(matches!(
        harness.actions().as_slice(),
        [PendingEditsAction::CancelClose]
    ));
}

#[test]
fn no_dialog_while_counting_finds_nothing_so_far() {
    let harness = Harness::new();
    let pending_edits = PendingEdits {
        is_counting: true,
        ..no_pending_edits()
    };
    assert_eq!(None, harness.run_dialog_frame(Vec::new(), &pending_edits));
    assert!(
        PendingEdits {
            is_counting: true,
            ..modified_tracks(1)
        }
        .is_close_dialog_needed()
    );
    assert!(
        PendingEdits {
            modified_track_count: None,
            ..no_pending_edits()
        }
        .is_close_dialog_needed()
    );
}

/// Click along the rows of the dialog from bottom to top until hitting the row of buttons.
fn click_buttons(pending_edits: &PendingEdits) -> Vec<&'static str> {
    let harness = Harness::new();
    harness.run_dialog_frame(Vec::new(), pending_edits);
    let rect = harness
        .run_dialog_frame(Vec::new(), pending_edits)
        .expect("rendered");
    let mut clicked = Vec::new();
    let mut y = rect.bottom() - 2.0;
    while clicked.is_empty() && y > rect.top() {
        let mut x = rect.left() + 2.0;
        while x < rect.right() {
            for events in click_events(Pos2::new(x, y)) {
                harness.run_dialog_frame(events, pending_edits);
            }
            for action in harness.actions() {
                let name = match action {
                    PendingEditsAction::Save { .. } => "save",
                    PendingEditsAction::CloseWithoutWriting => "close",
                    PendingEditsAction::CancelClose => "cancel",
                    PendingEditsAction::RequestRevert => "revert",
                    PendingEditsAction::RequestClose
                    | PendingEditsAction::CancelRevert
                    | PendingEditsAction::ConfirmRevert => "unexpected",
                };
                if clicked.last() != Some(&name) {
                    clicked.push(name);
                }
            }
            x += 8.0;
        }
        y -= 4.0;
    }
    clicked
}

#[test]
fn dialog_buttons_send_their_actions() {
    assert_eq!(
        vec!["save", "revert", "close", "cancel"],
        click_buttons(&modified_tracks(1))
    );
}

#[test]
fn only_cancel_is_available_while_writing() {
    assert_eq!(
        vec!["cancel"],
        click_buttons(&PendingEdits {
            is_writing: true,
            ..modified_tracks(1)
        })
    );
}

#[test]
fn invalid_draft_could_only_be_discarded() {
    assert_eq!(
        vec!["revert", "close", "cancel"],
        click_buttons(&PendingEdits {
            draft: DraftEdit::Invalid,
            ..modified_tracks(1)
        })
    );
}

#[test]
fn reverting_requires_pending_edits_and_an_idle_library() {
    assert!(!no_pending_edits().could_revert());
    assert!(modified_tracks(1).could_revert());
    // In contrast to saving an invalid draft could be reverted.
    assert!(
        PendingEdits {
            draft: DraftEdit::Invalid,
            ..no_pending_edits()
        }
        .could_revert()
    );
    assert!(
        PendingEdits {
            modified_track_count: None,
            ..no_pending_edits()
        }
        .could_revert()
    );
    assert!(
        !PendingEdits {
            is_writing: true,
            ..modified_tracks(1)
        }
        .could_revert()
    );
    assert!(
        !PendingEdits {
            could_write_files: false,
            ..modified_tracks(1)
        }
        .could_revert()
    );
}

#[test]
fn the_revert_button_shows_the_number_of_modified_tracks() {
    assert_eq!(
        "Revert pending edits",
        no_pending_edits().revert_button_text()
    );
    assert_eq!(
        "Revert pending edits (3)",
        modified_tracks(3).revert_button_text()
    );
}

/// The actions of the buttons in the confirmation dialog from left to right.
fn click_revert_dialog_buttons(pending_edits: &PendingEdits) -> Vec<&'static str> {
    let harness = Harness::new();
    let run_frame = |events: Vec<Event>| {
        let mut rect = None;
        let mut output = harness.ctx.run_ui(Harness::raw_input(events, false), |ui| {
            render_revert_dialog(ui.ctx(), &harness.msg_tx, pending_edits);
            rect = ui.ctx().memory(|mem| mem.area_rect(revert_dialog_id()));
        });
        output.textures_delta.clear();
        rect
    };
    run_frame(Vec::new());
    let rect = run_frame(Vec::new()).expect("rendered");
    let mut clicked = Vec::new();
    let mut y = rect.bottom() - 2.0;
    while clicked.is_empty() && y > rect.top() {
        let mut x = rect.left() + 2.0;
        while x < rect.right() {
            for events in click_events(Pos2::new(x, y)) {
                run_frame(events);
            }
            for action in harness.actions() {
                let name = match action {
                    PendingEditsAction::ConfirmRevert => "confirm",
                    PendingEditsAction::CancelRevert => "cancel",
                    _ => "unexpected",
                };
                if clicked.last() != Some(&name) {
                    clicked.push(name);
                }
            }
            x += 8.0;
        }
        y -= 4.0;
    }
    clicked
}

#[test]
fn revert_dialog_buttons_send_their_actions() {
    assert_eq!(
        vec!["confirm", "cancel"],
        click_revert_dialog_buttons(&modified_tracks(1))
    );
}

#[test]
fn reverting_could_not_be_confirmed_while_writing() {
    assert_eq!(
        vec!["cancel"],
        click_revert_dialog_buttons(&PendingEdits {
            is_writing: true,
            ..modified_tracks(1)
        })
    );
}
