use crate::auth::{decode_token, Auth};
use crate::config;
use crate::db;
use bytes::BufMut;
use chrono::Utc;
use futures::TryStreamExt;
use serde_json::json;
use std::net::SocketAddr;
use uuid::Uuid;
use warp::{
    //    http::StatusCode,
    multipart::{FormData, Part},
    reject,
    Filter,
    Rejection,
    Reply,
};

mod articles;
mod profiles;
mod tags;
mod users;

pub async fn web_routes() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "User-Agent",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Content-Type",
            "Authorization",
            "Accept",
            "Upgrade-Insecure-Requests",
            "Host",
            "Accept-Language",
            "Accept-Encoding",
            "Connection",
        ])
        .allow_methods(vec!["POST", "GET", "PUT", "DELETE"]);

    let index = warp::get()
        .and(warp::path::end())
        // .map(|| warp::redirect(Uri::from_static("/static/index.html")));
        .and(warp::fs::file(
            config::STATIC_PATH.to_string() + "index.html",
        ));
    let index1 = warp::get()
        .and(warp::path("index.html")) //not use path!, because don't wanna call end()
        // .map(|| warp::redirect(Uri::from_static("/static/index.html")));
        .and(warp::fs::file(
            config::STATIC_PATH.to_string() + "index.html",
        ));

    let board_mode = warp::get()
        .and(warp::path!("js" / "board_mode.js"))
        .and_then(handle_board_mode);
    let upload = warp::post()
        .and(warp::path("uploadFile"))
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::multipart::form().max_length(20_000_000))
        .and_then(handle_upload);

    let statics = warp::get()
        .and(warp::path("static"))
        .and(warp::fs::dir(config::STATIC_PATH));
    let downloads = warp::get()
        .and(warp::path("download"))
        .and(warp::fs::dir(config::DOWNLOAD_PATH));
    /*
    let testpath = warp::path("test")
        .and(warp::path("index.html"))
        .and(warp::path::tail())
        .map(|tail: warp::path::Tail| {
            format!("testpath: {:?}", tail)
        });*/
    let apis = users::route_create_user()
        .or(users::route_user_login())
        .or(users::route_put_user())
        .or(users::route_get_user())
        .or(articles::route_new_article())
        .or(articles::route_get_articles_feed())
        .or(articles::route_get_article())
        .or(articles::route_get_articles())
        .or(articles::route_get_comments())
        .or(articles::route_update_article())
        .or(articles::route_new_comment())
        .or(articles::route_delete_comment())
        .or(articles::route_delete_article())
        .or(articles::route_favorite_article())
        .or(articles::route_unfavorite_article())
        .or(tags::route_get_tags())
        .or(profiles::route_get_profile())
        .or(profiles::route_follow())
        .or(profiles::route_unfollow());
    let routes = apis
        .or(index)
        .or(index1)
        // .or(indexs)
        // .or(testpath)
        .or(statics)
        .or(downloads)
        .or(upload)
        .or(board_mode)
        .with(&cors);

    let addr: SocketAddr = db::CONFIG.web_url.parse().unwrap();
    warp::serve(routes).run(addr).await;
}

// "Rejection: HTTP method not allowed" when Authorization head not found or decode error
pub fn extract_auth_head() -> impl Filter<Extract = (Auth,), Error = warp::Rejection> + Copy {
    // warp::header::<String>("authorization").and_then(|s: String| async move {
    warp::header::optional::<String>("authorization").and_then(|ops: Option<String>| async move {
        let s = match ops {
            Some(s) => s,
            None => String::from(""),
        };
        if s.starts_with(config::TOKEN_PREFIX) {
            let auth = decode_token(&s[config::TOKEN_PREFIX.len()..]);
            match auth {
                Some(mut auth) => {
                    if auth.exp < Utc::now().timestamp() {
                        auth.exp = 0;
                    }
                    log::debug!("auth head : {:?}", auth);
                    Ok(auth)
                }
                None => {
                    if db::CONFIG.public_board {
                        Ok(Auth {
                            exp: 0,
                            id: 0,
                            username: String::from(""),
                        })
                    } else {
                        Err(reject::not_found())
                    }
                }
            }
        } else {
            if db::CONFIG.public_board {
                Ok(Auth {
                    exp: 0,
                    id: 0,
                    username: String::from(""),
                })
            } else {
                Err(reject::not_found())
            }
        }
    })
}

async fn handle_upload(form: FormData) -> Result<impl Reply, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        warp::reject::reject()
    })?;

    db::UPLOAD_LIST.lock().unwrap().clear();

    for p in parts {
        let mp_filename = match p.filename() {
            Some(f) => String::from(f),
            None => Uuid::new_v4().to_string(),
        };
        if p.name() == "file" {
            let value = p
                .stream()
                .try_fold(Vec::new(), |mut vec, data| {
                    //IMPORTANT: crate bytes version must same as warp version
                    vec.put(data);
                    async move { Ok(vec) }
                })
                .await
                .map_err(|e| {
                    log::error!("reading file error: {}", e);
                    warp::reject::reject()
                })?;

            let file_name = format!("{}{}", config::UPLOAD_DIR, mp_filename);
            tokio::fs::write(&file_name, value).await.map_err(|e| {
                log::error!("error writing file: {}", e);
                warp::reject::reject()
            })?;
            log::info!("created file: {}", file_name);
            db::UPLOAD_LIST.lock().unwrap().push(mp_filename);
        }
    }
    let files = db::UPLOAD_LIST.lock().unwrap().clone();
    let tmpjson = json!({ "status": "success", "filenames":files });
    Ok(warp::reply::json(&tmpjson))
}

async fn handle_board_mode() -> Result<impl Reply, Rejection> {
    Ok(format!("window.board_mode={};\n", db::CONFIG.public_board))
}
