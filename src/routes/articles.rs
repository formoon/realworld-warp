use crate::auth::Auth;
use crate::db;
use crate::db::articles::{FeedArticles, FindArticles};
use crate::errors::{Errors, FieldValidator};
use crate::routes;
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;
use validator::Validate;
use warp::http::StatusCode;
use warp::Filter;

#[derive(Deserialize, Debug)]
pub struct NewArticle {
    article: NewArticleData,
}

#[derive(Deserialize, Validate, Debug)]
pub struct NewArticleData {
    #[validate(length(min = 1))]
    title: Option<String>,
    #[validate(length(min = 1))]
    description: Option<String>,
    #[validate(length(min = 1))]
    body: Option<String>,
    #[serde(rename = "tagList")]
    tag_list: Vec<String>,
}
pub fn route_new_article(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "articles")
        .and(warp::post())
        .and(warp::body::json())
        .and(routes::extract_auth_head())
        .and_then(handler_new_article)
}
pub async fn handler_new_article(
    new_article: NewArticle,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_new_article: {:?}", new_article);
    if auth.exp == 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    }
    let new_article = new_article.article;

    let mut extractor = FieldValidator::validate(&new_article);
    let title = extractor.extract("title", new_article.title);
    let description = extractor.extract("description", new_article.description);
    let body = extractor.extract("body", new_article.body);
    match extractor.check() {
        Ok(_) => (),
        Err(err) => return err.respond_to(),
    };

    let conn = db::PG_POOL.clone().get().unwrap();
    let article = db::articles::create(
        &conn,
        auth.id,
        &title,
        &description,
        &body,
        &new_article.tag_list,
    );
    let tmpjson = json!({ "article": article });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

// --------------------------------------------------------------------------------------------------------
/// return multiple articles, ordered by most recent first
pub fn route_get_articles(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("api" / "articles"))
        // .and(warp::path::end())
        .and(warp::query::<FindArticles>())
        .and(routes::extract_auth_head())
        .and_then(handler_get_articles)
}
pub async fn handler_get_articles(
    params: FindArticles,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_get_articles: {:?}", params);
    if auth.exp == 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    let user_id = Some(auth.id);
    let articles = db::articles::find(&conn, &params, user_id);
    let tmpjson = json!({ "articles": articles.0, "articlesCount": articles.1 });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

// --------------------------------------------------------------------------------------------------------
pub fn route_get_article(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("api" / "articles" / String))
        // .and(warp::path("api")).and(warp::path("articles"))
        // .and(warp::path::param())
        .and(routes::extract_auth_head())
        .and_then(handler_get_article)
}
pub async fn handler_get_article(slug: String, auth: Auth) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_get_article: {:?}", slug);
    // id == 0 means not login, but public_board, allow reading before login
    // exp == 0 means not login
    if auth.exp == 0 && auth.id != 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    let user_id = Some(auth.id);
    let re = db::articles::find_one(&conn, &slug, user_id);
    match re {
        Some(article) => {
            let tmpjson = json!({ "article": article });
            Ok(warp::reply::with_status(
                warp::reply::json(&tmpjson),
                StatusCode::OK,
            ))
        }
        None => {
            let err = Errors::new(&[("Article ", "Not found!")]);
            return err.respond_to();
        }
    }
}

// --------------------------------------------------------------------------------------------------------
pub fn route_get_comments(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("api" / "articles" / String / "comments"))
        .and(routes::extract_auth_head())
        .and_then(handler_get_comments)
}
pub async fn handler_get_comments(
    slug: String,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_get_comments: {:?}", slug);
    if auth.exp == 0 && auth.id != 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    let comments = db::comments::find_by_slug(&conn, &slug);
    let tmpjson = json!({ "comments": comments });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

// --------------------------------------------------------------------------------------------------------
#[derive(Deserialize, Debug)]
pub struct UpdateArticle {
    article: db::articles::UpdateArticleData,
}

pub fn route_update_article(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::put()
        .and(warp::path!("api" / "articles" / String))
        .and(warp::body::json())
        .and(routes::extract_auth_head())
        .and_then(handler_update_article)
}
pub async fn handler_update_article(
    slug: String,
    article: UpdateArticle,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_update_article: {:?}", article);
    if auth.exp == 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    let re = db::articles::update(&conn, &slug, auth.id, article.article);
    match re {
        Some(article) => {
            let tmpjson = json!({ "article": article });
            Ok(warp::reply::with_status(
                warp::reply::json(&tmpjson),
                StatusCode::OK,
            ))
        }
        None => {
            let err = Errors::new(&[("Article ", "Update Error!")]);
            return err.respond_to();
        }
    }
}

// --------------------------------------------------------------------------------------------------------
#[derive(Deserialize, Debug)]
pub struct NewComment {
    comment: NewCommentData,
}

#[derive(Deserialize, Validate, Debug)]
pub struct NewCommentData {
    #[validate(length(min = 1))]
    body: Option<String>,
}

pub fn route_new_comment(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("api" / "articles" / String / "comments"))
        .and(warp::body::json())
        .and(routes::extract_auth_head())
        .and_then(handler_new_comment)
}
pub async fn handler_new_comment(
    slug: String,
    new_comment: NewComment,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_new_comment: {:?}", new_comment);
    if auth.exp == 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    };
    let new_comment = new_comment.comment;
    let mut extractor = FieldValidator::validate(&new_comment);
    let body = extractor.extract("body", new_comment.body);
    match extractor.check() {
        Ok(_) => (),
        Err(err) => return err.respond_to(),
    };

    let conn = db::PG_POOL.clone().get().unwrap();
    let comment = db::comments::create(&conn, auth.id, &slug, &body);
    let tmpjson = json!({ "comment": comment });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}
// --------------------------------------------------------------------------------------------------------
pub fn route_delete_comment(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::delete()
        .and(warp::path!("api" / "articles" / String / "comments" / i32))
        .and(routes::extract_auth_head())
        .and_then(handler_delete_comment)
}
pub async fn handler_delete_comment(
    slug: String,
    id: i32,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_delete_comment: {:?}", slug);

    if auth.exp == 0 {
        // let err=Errors::new(&[("Login ", "Login Expired!")]);
        return Ok(StatusCode::FORBIDDEN);
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    db::comments::delete(&conn, auth.id, &slug, id);
    Ok(StatusCode::OK)
}

// --------------------------------------------------------------------------------------------------------
pub fn route_delete_article(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::delete()
        .and(warp::path!("api" / "articles" / String))
        .and(routes::extract_auth_head())
        .and_then(handler_delete_article)
}
pub async fn handler_delete_article(
    slug: String,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_delete_article: {:?}", slug);

    if auth.exp == 0 {
        // let err=Errors::new(&[("Login ", "Login Expired!")]);
        return Ok(StatusCode::FORBIDDEN);
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    db::articles::delete(&conn, &slug, auth.id);
    Ok(StatusCode::OK)
}

// --------------------------------------------------------------------------------------------------------
pub fn route_favorite_article(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("api" / "articles" / String / "favorite"))
        .and(routes::extract_auth_head())
        .and_then(handler_favorite_article)
}
pub async fn handler_favorite_article(
    slug: String,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_favorite_article: {:?}", slug);

    if auth.exp == 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    let re = db::articles::favorite(&conn, &slug, auth.id);
    match re {
        Some(article) => {
            let tmpjson = json!({ "article": article });
            Ok(warp::reply::with_status(
                warp::reply::json(&tmpjson),
                StatusCode::OK,
            ))
        }
        None => {
            let err = Errors::new(&[("Favorite Article ", "Favorite Article Error!")]);
            return err.respond_to();
        }
    }
}
// --------------------------------------------------------------------------------------------------------
pub fn route_unfavorite_article(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::delete()
        .and(warp::path!("api" / "articles" / String / "favorite"))
        .and(routes::extract_auth_head())
        .and_then(handler_unfavorite_article)
}
pub async fn handler_unfavorite_article(
    slug: String,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_unfavorite_article: {:?}", slug);

    if auth.exp == 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    let re = db::articles::unfavorite(&conn, &slug, auth.id);
    match re {
        Some(article) => {
            let tmpjson = json!({ "article": article });
            Ok(warp::reply::with_status(
                warp::reply::json(&tmpjson),
                StatusCode::OK,
            ))
        }
        None => {
            let err = Errors::new(&[("UnFavorite Article ", "UnFavorite Article Error!")]);
            return err.respond_to();
        }
    }
}
// --------------------------------------------------------------------------------------------------------
// #[get("/articles/feed?<params..>")]
pub fn route_get_articles_feed(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("api" / "articles" / "feed"))
        .and(warp::query::<FeedArticles>())
        .and(routes::extract_auth_head())
        .and_then(handler_get_articles_feed)
}
pub async fn handler_get_articles_feed(
    params: FeedArticles,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_get_articles_feed: {:?}", params);

    if auth.exp == 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    let articles = db::articles::feed(&conn, &params, auth.id);
    let articles_count = articles.len();
    let tmpjson = json!({ "articles": articles, "articlesCount": articles_count });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}
