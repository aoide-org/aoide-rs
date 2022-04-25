use seed::{prelude::*, *};

use aoide_core::media::content::ContentPathConfig;

use crate::{domain::*, Action, Mdl, Msg};

pub(crate) fn view(mdl: &Mdl) -> Node<Msg> {
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
            ul![mdl.collections.values().map(|item| { collection(item) })]
        }
    ]
}

fn collection(item: &CollectionItem) -> Node<Msg> {
    let CollectionItem { entity, summary } = item;
    let uid = entity.hdr.uid.to_owned();
    li![div![
        C!["card"],
        header![
            C!["card-header"],
            p![C!["card-header-title"], &entity.body.title],
        ],
        div![
            C!["card-content"],
            div![
                C!["content"],
                ul![
                    if let Some(kind) = &entity.body.kind {
                        li![b!["Kind: "], kind]
                    } else {
                        empty![]
                    },
                    if let Some(notes) = &entity.body.notes {
                        li![b!["Notes: "], notes]
                    } else {
                        empty![]
                    },
                    match &entity.body.media_source_config.content_path {
                        ContentPathConfig::VirtualFilePath { root_url } =>
                            li![b!["VFS Root: "], format!("{}", root_url)],
                        _ => empty![],
                    },
                    if let Some(summary) = summary {
                        // TODO: Display detailed
                        li![b!["Summary: "], format!("{:?}", summary)]
                    } else {
                        empty![]
                    }
                ]
            ]
        ],
        footer![
            C!["card-footer"],
            a![
                C!["card-footer-item"],
                ev(Ev::Click, |_| Msg::Action(Action::LoadCollection(uid))),
                "Load",
            ]
        ]
    ]]
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
