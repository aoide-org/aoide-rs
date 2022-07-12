// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use rust_embed::RustEmbed;
use warp::{
    filters::BoxedFilter, http::HeaderValue, hyper::header::CONTENT_TYPE, path::Tail, Filter as _,
    Rejection, Reply,
};

#[derive(RustEmbed)]
#[folder = "../webapp/dist/"]
struct Asset;

pub(crate) fn get_index() -> BoxedFilter<(impl Reply,)> {
    let index = warp::path("index.html")
        .and(warp::path::end())
        .or(warp::path::end());
    warp::get().and(index).and_then(|_| serve_index()).boxed()
}

pub(crate) fn get_assets() -> BoxedFilter<(impl Reply,)> {
    warp::get().and(warp::path::tail()).and_then(serve).boxed()
}

async fn serve_index() -> Result<impl Reply, Rejection> {
    serve_impl("index.html")
}

async fn serve(path: Tail) -> Result<impl Reply, Rejection> {
    serve_impl(path.as_str())
}

fn serve_impl(path: &str) -> Result<impl Reply, Rejection> {
    let asset = Asset::get(path).ok_or_else(warp::reject::not_found)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let mut res = warp::reply::Response::new(asset.data.into());
    match HeaderValue::from_str(mime.as_ref()) {
        Ok(mime) => {
            res.headers_mut().insert(CONTENT_TYPE, mime);
        }
        Err(_) => {
            log::warn!("Unexpected content type: {mime}");
        }
    }
    Ok(res)
}
