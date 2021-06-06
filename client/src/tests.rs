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

use std::sync::Arc;

use reqwest::Url;

use crate::{
    collection, handle_messages, handle_next_message, media::tracker as media_tracker, prelude::*,
    Effect, Intent, Message, MessageLoopControl, State,
};

fn dummy_api_url() -> Url {
    "http://[::1]:8080".parse().unwrap()
}

fn test_env() -> Environment {
    Environment::new(dummy_api_url())
}

#[test]
fn should_handle_error() {
    let (message_tx, _) = message_channel::<Message>();
    let shared_env = Arc::new(test_env());
    let mut state = State::default();
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert_eq!(
        MessageLoopControl::Terminate,
        handle_next_message(
            &shared_env,
            Some(&message_tx),
            &mut state,
            &mut |_| { None },
            effect.into()
        )
    );
    assert_eq!(1, state.last_errors().len());
}

#[tokio::test]
async fn should_catch_error() {
    let shared_env = Arc::new(test_env());
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    let state = handle_messages(
        shared_env,
        Default::default(),
        Intent::InjectEffect(Box::new(effect)),
        Box::new(|_| None),
    )
    .await;
    assert_eq!(1, state.last_errors().len());
}

#[test]
fn should_handle_collection_error() {
    let (message_tx, _) = message_channel::<Message>();
    let shared_env = Arc::new(test_env());
    let mut state = State::default();
    let effect = collection::Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert_eq!(
        MessageLoopControl::Terminate,
        handle_next_message(
            &shared_env,
            Some(&message_tx),
            &mut state,
            &mut |_| { None },
            effect.into()
        )
    );
    assert_eq!(1, state.last_errors().len());
}

#[tokio::test]
async fn should_catch_collection_error() {
    let shared_env = Arc::new(test_env());
    let effect = collection::Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    let state = handle_messages(
        shared_env,
        Default::default(),
        Intent::InjectEffect(Box::new(effect.into())),
        Box::new(|_| None),
    )
    .await;
    assert_eq!(1, state.last_errors().len());
}

#[test]
fn should_handle_media_tracker_error() {
    let (message_tx, _) = message_channel::<Message>();
    let shared_env = Arc::new(test_env());
    let mut state = State::default();
    let effect = media_tracker::Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert_eq!(
        MessageLoopControl::Terminate,
        handle_next_message(
            &shared_env,
            Some(&message_tx),
            &mut state,
            &mut |_| { None },
            effect.into()
        )
    );
    assert_eq!(1, state.last_errors().len());
}

#[tokio::test]
async fn should_catch_media_tracker_error() {
    let shared_env = Arc::new(test_env());
    let effect = media_tracker::Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    let state = handle_messages(
        shared_env,
        Default::default(),
        Intent::InjectEffect(Box::new(effect.into())),
        Box::new(|_| None),
    )
    .await;
    assert_eq!(1, state.last_errors().len());
}
