use crate::auth::Auth;
use crate::db;
use crate::errors::Errors;
use crate::models::user::Profile;
use crate::routes;
use serde_json::json;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::Filter;

fn to_profile_json(
    profile: Profile,
) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    let tmpjson = json!({ "profile": profile });
    Ok(warp::reply::with_status(
        warp::reply::json(&tmpjson),
        StatusCode::OK,
    ))
}

//#[get("/profiles/<username>")]
pub fn route_get_profile(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("api" / "profiles" / String))
        .and(routes::extract_auth_head())
        .and_then(handler_get_profile)
}
pub async fn handler_get_profile(
    username: String,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_get_profile: {:?}", username);
    let user_id = Some(auth.id);
    if auth.exp == 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    let re = db::profiles::find(&conn, &username, user_id);
    match re {
        Some(re) => to_profile_json(re),
        None => {
            let err = Errors::new(&[("handler_get_profile ", "handler_get_profile Error!")]);
            return err.respond_to();
        }
    }
}
// --------------------------------------------------------------------------------------------------------
// #[post("/profiles/<username>/follow")]
pub fn route_follow() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("api" / "profiles" / String / "follow"))
        .and(routes::extract_auth_head())
        .and_then(handler_follow)
}
pub async fn handler_follow(username: String, auth: Auth) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_follow: {:?}", username);

    if auth.exp == 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    let re = db::profiles::follow(&conn, &username, auth.id);
    match re {
        Some(re) => to_profile_json(re),
        None => {
            let err = Errors::new(&[("handler_follow ", "handler_follow Error!")]);
            return err.respond_to();
        }
    }
}

// --------------------------------------------------------------------------------------------------------
// #[delete("/profiles/<username>/follow")]
pub fn route_unfollow() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::delete()
        .and(warp::path!("api" / "profiles" / String / "follow"))
        .and(routes::extract_auth_head())
        .and_then(handler_unfollow)
}
pub async fn handler_unfollow(
    username: String,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_unfollow: {:?}", username);

    if auth.exp == 0 {
        let err = Errors::new(&[("Login ", "Login Expired!")]);
        return err.respond_to();
    };
    let conn = db::PG_POOL.clone().get().unwrap();
    let re = db::profiles::unfollow(&conn, &username, auth.id);
    match re {
        Some(re) => to_profile_json(re),
        None => {
            let err = Errors::new(&[("handler_unfollow ", "handler_unfollow Error!")]);
            return err.respond_to();
        }
    }
}
