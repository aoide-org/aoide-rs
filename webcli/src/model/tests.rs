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

///////////////////////////////////////////////////////////////////////

use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::{Duration, Instant},
};

use reqwest::Url;

use aoide_client::{
    messaging::{
        handle_next_message, message_channel, message_loop, send_message, MessageHandled,
        TaskDispatcher as _,
    },
    models::media_tracker,
};

use super::*;

fn dummy_api_url() -> Url {
    "http://[::1]:8080".parse().unwrap()
}

fn test_env() -> Environment {
    Environment::new(dummy_api_url())
}

#[test]
fn should_handle_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, _) = message_channel();
    let mut state = State::default();
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert_eq!(
        MessageHandled::NoProgress,
        handle_next_message(
            &shared_env,
            &mut state,
            &message_tx,
            effect.into(),
            &mut |_| { None },
        )
    );
    assert_eq!(1, state.last_errors().len());
}

#[tokio::test]
async fn should_catch_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, message_rx) = message_channel();
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    send_message(&message_tx, Intent::InjectEffect(Box::new(effect)));
    let state = message_loop(
        shared_env,
        (message_tx, message_rx),
        Default::default(),
        Box::new(|_: &State| None),
    )
    .await;
    assert_eq!(1, state.last_errors().len());
}

#[test]
fn should_handle_collection_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, _) = message_channel();
    let mut state = State::default();
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert_eq!(
        MessageHandled::NoProgress,
        handle_next_message(
            &shared_env,
            &mut state,
            &message_tx,
            effect.into(),
            &mut |_| { None },
        )
    );
    assert_eq!(1, state.last_errors().len());
}

#[tokio::test]
async fn should_catch_collection_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, message_rx) = message_channel();
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    send_message(&message_tx, Intent::InjectEffect(Box::new(effect)));
    let state = message_loop(
        shared_env,
        (message_tx, message_rx),
        Default::default(),
        Box::new(|_: &State| None),
    )
    .await;
    assert_eq!(1, state.last_errors().len());
}

#[test]
fn should_handle_media_tracker_error() {
    let shared_env = Arc::new(test_env());
    let (message_tx, _) = message_channel();
    let mut state = State::default();
    let effect = media_tracker::Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert_eq!(
        MessageHandled::NoProgress,
        handle_next_message(
            &shared_env,
            &mut state,
            &message_tx,
            Effect::MediaTracker(effect).into(),
            &mut |_| { None },
        )
    );
    assert_eq!(1, state.last_errors().len());
}

#[tokio::test]
async fn should_catch_media_tracker_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, message_rx) = message_channel();
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    send_message(&message_tx, Intent::InjectEffect(Box::new(effect)));
    let state = message_loop(
        shared_env,
        (message_tx, message_rx),
        Default::default(),
        Box::new(|_: &State| None),
    )
    .await;
    assert_eq!(1, state.last_errors().len());
}

#[tokio::test]
async fn should_terminate_on_intent_when_no_tasks_pending() {
    let shared_env = Arc::new(test_env());
    let (message_tx, message_rx) = message_channel();
    send_message(&message_tx, Intent::Terminate);
    let state = message_loop(
        shared_env,
        (message_tx, message_rx),
        Default::default(),
        Box::new(|_: &State| None),
    )
    .await;
    assert!(state.last_errors().is_empty());
}

#[tokio::test]
async fn should_terminate_on_intent_after_pending_tasks_finished() {
    let shared_env = Arc::new(test_env());
    let (message_tx, message_rx) = message_channel();
    send_message(
        &message_tx,
        Intent::Deferred {
            not_before: Instant::now() + Duration::from_millis(100),
            intent: Box::new(Intent::RenderState),
        },
    );
    send_message(&message_tx, Intent::Terminate);
    let render_state_count = Arc::new(AtomicUsize::new(0));
    let state = message_loop(
        shared_env.clone(),
        (message_tx, message_rx),
        Default::default(),
        Box::new({
            let shared_env = Arc::clone(&shared_env);
            let render_state_count = Arc::clone(&render_state_count);
            move |state: &State| {
                let last_render_state_count =
                    render_state_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                // On the 1st (initial) and 2nd (Intent::Terminate) invocation the task
                // that executes the timed intent is still pending
                assert_eq!(
                    last_render_state_count == 0,
                    state.control_state == state::ControlState::Running
                );
                assert_eq!(
                    last_render_state_count > 0,
                    state.control_state == state::ControlState::Terminating
                );
                assert_eq!(last_render_state_count > 1, shared_env.all_tasks_finished());
                None
            }
        }),
    )
    .await;
    assert_eq!(
        3,
        render_state_count.load(std::sync::atomic::Ordering::SeqCst)
    );
    assert!(state.last_errors().is_empty());
}
