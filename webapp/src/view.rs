use crate::{domain::*, Mdl, Msg};
use seed::{prelude::*, *};

pub fn view(mdl: &Mdl) -> Node<Msg> {
    div![
        error_msg(&mdl.error),
        section![
            C!["section"],
            div![C!["container"], header(), collections(mdl),]
        ]
    ]
}

fn header<M>() -> Node<M> {
    div![
        C!["block"],
        h1![C!["title", "is-1"], "aoide"],
        p![C!["subtitle"], "Managing and exploring music collections"]
    ]
}

fn collections(mdl: &Mdl) -> Node<Msg> {
    div![
        h3![C!["title", "is-3"], "Collections"],
        if mdl.collections.is_empty() {
            p!["There currently are no collections"]
        } else {
            ul![mdl.collections.iter().map(|(id, c)| { collection(id, c) })]
        }
    ]
}

fn collection(id: &str, c: &CollectionData) -> Node<Msg> {
    match c {
        CollectionData::Overview(c) => {
            let id = id.to_string();
            li![div![
                C!["card"],
                header![C!["card-header"], p![C!["card-header-title"], &c.title],],
                div![
                    C!["card-content"],
                    div![
                        C!["content"],
                        ul![
                            if let Some(n) = &c.notes {
                                li![b!["Notes: "], n]
                            } else {
                                empty![]
                            },
                            if let Some(k) = &c.kind {
                                li![b!["Kind: "], k]
                            } else {
                                empty![]
                            },
                            if let Some(url) = &c.media_source_config.source_path.root_url {
                                li![b!["Root: "], format!("{}", url)]
                            } else {
                                empty![]
                            },
                        ]
                    ]
                ],
                footer![
                    C!["card-footer"],
                    a![
                        C!["card-footer-item"],
                        ev(Ev::Click, |_| Msg::LoadCollection(id)),
                        "Load",
                    ]
                ]
            ]]
        }
        CollectionData::WithSummary(_) => {
            empty![
              // TODO
            ]
        }
    }
}

fn error_msg(msg: &Option<String>) -> Node<Msg> {
    if let Some(msg) = &msg {
        div![
            style! {
              St::AlignItems => "center";
              St::Display => "flex";
              St::JustifyContent => "center";
              St::Padding =>  em(0.5);
              St::FontSize =>  rem(0.875);
              St::Color => "#f14668";
              St::BackgroundColor => "#fee";
            },
            p![msg]
        ]
    } else {
        empty![]
    }
}
