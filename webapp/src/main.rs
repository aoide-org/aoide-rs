#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::all)]
#![deny(clippy::explicit_deref_methods)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::must_use_candidate)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

use std::collections::HashMap;

use seed::prelude::*;

use aoide_core::entity::EntityUid;

mod api;
mod domain;
mod view;

use domain::*;

// ------ ------
//     Model
// ------ ------

#[derive(Debug, Default)]
pub struct Mdl {
    collections: HashMap<EntityUid, CollectionItem>,
    error: Option<String>,
}

// ------ ------
//    Message
// ------ ------

#[derive(Debug)]
pub(crate) enum Msg {
    Action(Action),
    Event(Event),
    ApiError(api::Error),
}

#[derive(Debug)]
pub(crate) enum Action {
    LoadCollection(EntityUid),
}

impl From<Action> for Msg {
    fn from(action: Action) -> Self {
        Self::Action(action)
    }
}

#[derive(Debug)]
pub(crate) enum Event {
    AllCollectionsFetched(Box<CollectionItems>),
    CollectionWithSummaryFetched(Box<CollectionItem>),
}

impl From<Event> for Msg {
    fn from(event: Event) -> Self {
        Self::Event(event)
    }
}

// ------ ------
//    Update
// ------ ------

fn update(msg: Msg, mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
    seed::log!(msg);
    match msg {
        Msg::Action(action) => match action {
            Action::LoadCollection(uid) => {
                orders.perform_cmd(async {
                    api::fetch_collection_with_summary(uid)
                        .await
                        .map(Box::new)
                        .map(Event::CollectionWithSummaryFetched)
                        .map(Msg::Event)
                        .unwrap_or_else(Msg::ApiError)
                });
            }
        },
        Msg::Event(event) => match event {
            Event::AllCollectionsFetched(items) => {
                mdl.collections = items
                    .into_iter()
                    .map(|item| (item.entity.hdr.uid.clone(), item))
                    .collect();
            }
            Event::CollectionWithSummaryFetched(item) => {
                debug_assert!(item.summary.is_some());
                mdl.collections.insert(item.entity.hdr.uid.clone(), *item);
            }
        },
        Msg::ApiError(err) => {
            mdl.error = Some(format!("{}", err));
        }
    }
}

// ------ ------
//    View
// ------ ------

fn view(mdl: &Mdl) -> Node<Msg> {
    view::view(mdl)
}

// ------ ------
//     Init
// ------ ------

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Mdl {
    orders.perform_cmd(async {
        api::fetch_all_collections()
            .await
            .map(Box::new)
            .map(Event::AllCollectionsFetched)
            .map(Msg::Event)
            .unwrap_or_else(Msg::ApiError)
    });
    Mdl::default()
}

// ------ ------
//     Start
// ------ ------

fn main() {
    App::start("app", init, update, view);
}
