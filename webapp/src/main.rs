use seed::prelude::*;
use std::collections::HashMap;

mod api;
mod domain;
mod view;

use domain::*;

// ------ ------
//     Model
// ------ ------

#[derive(Default)]
pub struct Mdl {
    collections: HashMap<CollectionId, CollectionData>,
    error: Option<String>,
}

// ------ ------
//    Message
// ------ ------

#[derive(Debug)]
pub enum Msg {
    LoadCollection(CollectionId),
    CollectionsFetched(Box<Collections>),
    CollectionWithSummaryFetched(CollectionId, Box<CollectionWithSummary>),
    ApiError(api::Error),
}

// ------ ------
//    Update
// ------ ------

fn update(msg: Msg, mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
    seed::log!(msg);
    match msg {
        Msg::CollectionsFetched(collections) => {
            collections.into_iter().for_each(|((id, _), c)| {
                mdl.collections.insert(id, CollectionData::Overview(c));
            });
        }
        Msg::CollectionWithSummaryFetched(id, c) => {
            mdl.collections.insert(id, CollectionData::WithSummary(*c));
        }
        Msg::LoadCollection(id) => {
            orders.perform_cmd(async {
                match api::get_collection(&id).await {
                    Ok(c) => Msg::CollectionWithSummaryFetched(id, Box::new(c)),
                    Err(e) => Msg::ApiError(e),
                }
            });
        }
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
        match api::get_all_collections().await {
            Ok(c) => Msg::CollectionsFetched(Box::new(c)),
            Err(e) => Msg::ApiError(e),
        }
    });
    Mdl::default()
}

// ------ ------
//     Start
// ------ ------

fn main() {
    App::start("app", init, update, view);
}
