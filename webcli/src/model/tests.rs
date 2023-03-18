// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::{Duration, Instant},
};

use infect::{
    message_channel, process_messages, process_next_message, send_message, NextMessageProcessed,
    ProcessingMessagesStopped, TaskContext,
};
use reqwest::Url;

use aoide_client::models::media_tracker;

use super::*;

fn dummy_api_url() -> Url {
    "http://[::1]:8080".parse().unwrap()
}

fn test_env() -> Environment {
    Environment::new(dummy_api_url())
}

const MESSAGE_CHANNEL_CAPACITY: usize = 10;

#[derive(Default)]
struct RenderModel;

impl infect::RenderModel for RenderModel {
    type Model = Model;

    fn render_model(
        &mut self,
        _model: &Self::Model,
    ) -> Option<<Self::Model as ClientModel>::Intent> {
        None
    }
}

#[test]
fn should_handle_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, _) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let mut task_context = TaskContext {
        message_tx,
        task_executor: shared_env,
    };
    let mut model = Model::default();
    let mut render_model = RenderModel;
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert!(matches!(
        process_next_message(
            &mut task_context,
            &mut model,
            &mut render_model,
            effect.into(),
        ),
        NextMessageProcessed::NoProgress,
    ));
    assert_eq!(1, model.last_errors().len());
}

#[tokio::test]
async fn should_catch_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (mut message_tx, mut message_rx) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let task_context = &mut TaskContext {
        message_tx: message_tx.clone(),
        task_executor: shared_env,
    };
    let model = &mut Model::default();
    let render_model = &mut RenderModel;
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    send_message(&mut message_tx, effect);
    process_messages(&mut message_rx, task_context, model, render_model).await;
    assert_eq!(1, model.last_errors().len());
}

#[test]
fn should_handle_collection_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, _) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let mut task_context = TaskContext {
        message_tx,
        task_executor: shared_env,
    };
    let mut model = Model::default();
    let mut render_model = RenderModel;
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert!(matches!(
        process_next_message(
            &mut task_context,
            &mut model,
            &mut render_model,
            effect.into(),
        ),
        NextMessageProcessed::NoProgress,
    ));
    assert_eq!(1, model.last_errors().len());
}

#[tokio::test]
async fn should_catch_collection_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (mut message_tx, mut message_rx) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let task_context = &mut TaskContext {
        message_tx: message_tx.clone(),
        task_executor: shared_env,
    };
    let model = &mut Model::default();
    let render_model = &mut RenderModel;
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    send_message(&mut message_tx, effect);
    process_messages(&mut message_rx, task_context, model, render_model).await;
    assert_eq!(1, model.last_errors().len());
}

#[test]
fn should_handle_media_tracker_error() {
    let shared_env = Arc::new(test_env());
    let (message_tx, _) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let task_context = &mut TaskContext {
        message_tx,
        task_executor: shared_env,
    };
    let model = &mut Model::default();
    let render_model = &mut RenderModel;
    let effect = media_tracker::Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert!(matches!(
        process_next_message(
            task_context,
            model,
            render_model,
            Effect::MediaTracker(effect).into(),
        ),
        NextMessageProcessed::NoProgress,
    ));
    assert_eq!(1, model.last_errors().len());
}

#[tokio::test]
async fn should_catch_media_tracker_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (mut message_tx, mut message_rx) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let task_context = &mut TaskContext {
        message_tx: message_tx.clone(),
        task_executor: shared_env,
    };
    let model = &mut Model::default();
    let render_model = &mut RenderModel;
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    send_message(&mut message_tx, effect);
    process_messages(&mut message_rx, task_context, model, render_model).await;
    assert_eq!(1, model.last_errors().len());
}

#[tokio::test]
async fn should_terminate_on_intent_when_no_tasks_pending() {
    let shared_env = Arc::new(test_env());
    let (mut message_tx, mut message_rx) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let task_context = &mut TaskContext {
        message_tx: message_tx.clone(),
        task_executor: shared_env,
    };
    let model = &mut Model::default();
    let render_model = &mut RenderModel;
    send_message(&mut message_tx, Intent::Terminate);
    process_messages(&mut message_rx, task_context, model, render_model).await;
    assert!(model.last_errors().is_empty());
}

struct TerminationRenderModel {
    shared_env: Arc<Environment>,
    invocation_count: Arc<AtomicUsize>,
}

impl TerminationRenderModel {
    fn new(shared_env: Arc<Environment>) -> Self {
        Self {
            shared_env,
            invocation_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl infect::RenderModel for TerminationRenderModel {
    type Model = Model;

    fn render_model(
        &mut self,
        model: &Self::Model,
    ) -> Option<<Self::Model as ClientModel>::Intent> {
        let last_invocation_count = self
            .invocation_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(State::Terminating, model.state);
        assert!(last_invocation_count < 2);
        assert_eq!(
            last_invocation_count == 0,
            !self.shared_env.all_tasks_finished()
        );
        assert_eq!(
            last_invocation_count == 1,
            self.shared_env.all_tasks_finished()
        );
        None
    }
}

#[tokio::test]
async fn should_terminate_on_intent_after_pending_tasks_finished() {
    let shared_env = Arc::new(test_env());
    let (mut message_tx, mut message_rx) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let task_context = &mut TaskContext {
        message_tx: message_tx.clone(),
        task_executor: Arc::clone(&shared_env),
    };
    let model = &mut Model::default();
    let render_model = &mut TerminationRenderModel::new(Arc::clone(&shared_env));
    send_message(
        &mut message_tx,
        Intent::Scheduled {
            not_before: Instant::now() + Duration::from_millis(100),
            intent: Box::new(Intent::RenderModel),
        },
    );
    send_message(&mut message_tx, Intent::Terminate);
    assert_eq!(model.state, State::Running);
    loop {
        let stopped = process_messages(&mut message_rx, task_context, model, render_model).await;
        assert_eq!(model.state, State::Terminating);
        assert!(matches!(stopped, ProcessingMessagesStopped::NoProgress));
        if shared_env.all_tasks_finished() {
            break;
        }
    }
    assert_eq!(
        2,
        render_model
            .invocation_count
            .load(std::sync::atomic::Ordering::SeqCst)
    );
    assert!(model.last_errors().is_empty());
}
