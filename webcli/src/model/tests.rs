// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::{Duration, Instant},
};

use aoide_client::models::media_tracker;
use infect::{
    consume_messages, message_channel, process_message, MessagePort, MessageProcessed,
    MessagesConsumed, TaskContext,
};
use reqwest::Url;

use super::*;

fn dummy_api_url() -> Url {
    "http://[::1]:8080".parse().unwrap()
}

fn test_env() -> Environment {
    Environment::new(dummy_api_url())
}

const MESSAGE_CHANNEL_CAPACITY: usize = 10;

#[derive(Default)]
struct ModelRender;

impl infect::ModelRender for ModelRender {
    type Model = Model;

    fn render_model(
        &mut self,
        _model: &Self::Model,
        model_changed: ModelChanged,
    ) -> Option<<Self::Model as ClientModel>::Intent> {
        assert_eq!(ModelChanged::MaybeChanged, model_changed);
        None
    }
}

#[test]
fn should_handle_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, _) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let message_port = MessagePort::new(message_tx);
    let mut task_context = TaskContext {
        message_port,
        task_executor: shared_env,
    };
    let mut model = Model::default();
    let mut model_render = ModelRender;
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert!(matches!(
        process_message(
            &mut task_context,
            &mut model,
            &mut model_render,
            effect.into(),
        ),
        MessageProcessed::NoProgress,
    ));
    assert_eq!(1, model.last_errors().count());
}

#[tokio::test]
async fn should_catch_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, mut message_rx) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let message_port = MessagePort::new(message_tx);
    let task_context = &mut TaskContext {
        message_port,
        task_executor: shared_env,
    };
    let model = &mut Model::default();
    let model_render = &mut ModelRender;
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    task_context.submit_effect(effect);
    consume_messages(&mut message_rx, task_context, model, model_render).await;
    assert_eq!(1, model.last_errors().count());
}

#[test]
fn should_handle_collection_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, _) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let message_port = MessagePort::new(message_tx);
    let mut task_context = TaskContext {
        message_port,
        task_executor: shared_env,
    };
    let mut model = Model::default();
    let mut model_render = ModelRender;
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert!(matches!(
        process_message(
            &mut task_context,
            &mut model,
            &mut model_render,
            effect.into(),
        ),
        MessageProcessed::NoProgress,
    ));
    assert_eq!(1, model.last_errors().count());
}

#[tokio::test]
async fn should_catch_collection_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, mut message_rx) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let message_port = MessagePort::new(message_tx);
    let task_context = &mut TaskContext {
        message_port,
        task_executor: shared_env,
    };
    let model = &mut Model::default();
    let model_render = &mut ModelRender;
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    task_context.submit_effect(effect);
    consume_messages(&mut message_rx, task_context, model, model_render).await;
    assert_eq!(1, model.last_errors().count());
}

#[test]
fn should_handle_media_tracker_error() {
    let shared_env = Arc::new(test_env());
    let (message_tx, _) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let message_port = MessagePort::new(message_tx);
    let task_context = &mut TaskContext {
        message_port,
        task_executor: shared_env,
    };
    let model = &mut Model::default();
    let model_render = &mut ModelRender;
    let effect = media_tracker::Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    assert!(matches!(
        process_message(
            task_context,
            model,
            model_render,
            Effect::MediaTracker(effect).into(),
        ),
        MessageProcessed::NoProgress,
    ));
    assert_eq!(1, model.last_errors().count());
}

#[tokio::test]
async fn should_catch_media_tracker_error_and_terminate() {
    let shared_env = Arc::new(test_env());
    let (message_tx, mut message_rx) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let message_port = MessagePort::new(message_tx);
    let task_context = &mut TaskContext {
        message_port,
        task_executor: shared_env,
    };
    let model = &mut Model::default();
    let model_render = &mut ModelRender;
    let effect = Effect::ErrorOccurred(anyhow::anyhow!("an error occurred"));
    task_context.message_port.submit_effect(effect);
    consume_messages(&mut message_rx, task_context, model, model_render).await;
    assert_eq!(1, model.last_errors().count());
}

#[tokio::test]
async fn should_terminate_on_intent_when_no_tasks_pending() {
    let shared_env = Arc::new(test_env());
    let (message_tx, mut message_rx) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let message_port = MessagePort::new(message_tx);
    let task_context = &mut TaskContext {
        message_port,
        task_executor: shared_env,
    };
    let model = &mut Model::default();
    let model_render = &mut ModelRender;
    task_context.submit_intent(Intent::Terminate);
    consume_messages(&mut message_rx, task_context, model, model_render).await;
    assert!(model.last_errors().next().is_none());
}

struct TerminationModelRender {
    shared_env: Arc<Environment>,
    invocation_count: Arc<AtomicUsize>,
}

impl TerminationModelRender {
    fn new(shared_env: Arc<Environment>) -> Self {
        Self {
            shared_env,
            invocation_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl infect::ModelRender for TerminationModelRender {
    type Model = Model;

    fn render_model(
        &mut self,
        model: &Self::Model,
        model_changed: ModelChanged,
    ) -> Option<<Self::Model as ClientModel>::Intent> {
        assert_eq!(ModelChanged::MaybeChanged, model_changed);
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
    let (message_tx, mut message_rx) = message_channel(MESSAGE_CHANNEL_CAPACITY);
    let message_port = MessagePort::new(message_tx);
    let task_context = &mut TaskContext {
        message_port,
        task_executor: Arc::clone(&shared_env),
    };
    let model = &mut Model::default();
    let model_render = &mut TerminationModelRender::new(Arc::clone(&shared_env));
    task_context.submit_intent(Intent::Schedule {
        not_before: Instant::now() + Duration::from_millis(100),
        intent: Box::new(Intent::RenderModel),
    });
    task_context.submit_intent(Intent::Terminate);
    assert_eq!(model.state, State::Running);
    loop {
        let stopped = consume_messages(&mut message_rx, task_context, model, model_render).await;
        assert_eq!(model.state, State::Terminating);
        assert!(matches!(stopped, MessagesConsumed::NoProgress));
        if shared_env.all_tasks_finished() {
            break;
        }
    }
    assert_eq!(
        2,
        model_render
            .invocation_count
            .load(std::sync::atomic::Ordering::SeqCst)
    );
    assert!(model.last_errors().next().is_none());
}
