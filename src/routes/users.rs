use crate::auth::Auth;
use crate::db::{self, users::UserCreationError};
use crate::errors::{Errors, FieldValidator};
use crate::routes;

use std::convert::Infallible;

use warp::http::StatusCode;
use warp::Filter;

use serde::Deserialize;
use serde_json::json;
use validator::Validate;

#[derive(Deserialize, Debug)]
pub struct NewUser {
    user: NewUserData,
}

#[derive(Debug, Deserialize, Validate)]
struct NewUserData {
    #[validate(length(min = 1))]
    username: Option<String>,
    #[validate(email)]
    email: Option<String>,
    #[validate(length(min = 8))]
    password: Option<String>,
}

pub fn route_create_user(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "users")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handler_create_user)
}
pub async fn handler_create_user(new_user: NewUser) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_create_user: {:?}", new_user);

    let new_user = new_user.user;

    let mut extractor = FieldValidator::validate(&new_user);
    let username = extractor.extract("username", new_user.username);
    let email = extractor.extract("email", new_user.email);
    let password = extractor.extract("password", new_user.password);

    match extractor.check() {
        Ok(_) => (),
        Err(err) => return err.respond_to(),
    };

    let conn = db::PG_POOL.clone().get().unwrap();
    let result = db::users::create(&conn, &username, &email, &password);
    match result {
        Ok(user) => {
            let tmpjson = json!({ "user": user.to_user_auth() });
            Ok(warp::reply::with_status(
                warp::reply::json(&tmpjson),
                StatusCode::OK,
            ))
        }
        Err(error) => {
            let field = match error {
                UserCreationError::DuplicatedEmail => "email",
                UserCreationError::DuplicatedUsername => "username",
            };
            let err = Errors::new(&[(field, "has already been taken")]);
            err.respond_to()
        }
    }
}

// --------------------------------------------------------------------------------------------------------
#[derive(Deserialize, Debug)]
pub struct LoginUser {
    user: LoginUserData,
}

#[derive(Deserialize, Debug)]
struct LoginUserData {
    email: Option<String>,
    password: Option<String>,
}

pub fn route_user_login() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path!("api" / "users" / "login")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handler_user_login)
}
pub async fn handler_user_login(user: LoginUser) -> Result<impl warp::Reply, Infallible> {
    let user = user.user;
    log::debug!("handler_user_login: {:?}", user);

    let mut extractor = FieldValidator::default();
    let email = extractor.extract("email", user.email);
    let password = extractor.extract("password", user.password);
    match extractor.check() {
        Ok(_) => (),
        Err(err) => return err.respond_to(),
    };

    let conn = db::PG_POOL.clone().get().unwrap();
    let result = db::users::login(&conn, &email, &password);
    match result {
        Some(user) => {
            // Ok(json!({ "user": user.to_user_auth() }))
            let tmpjson = json!({ "user": user.to_user_auth() });
            Ok(warp::reply::with_status(
                warp::reply::json(&tmpjson),
                StatusCode::OK,
            ))
        }
        None => {
            let err = Errors::new(&[("email or password", "is invalid")]);
            err.respond_to()
        }
    }
}

// --------------------------------------------------------------------------------------------------------
pub fn route_get_user() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path!("api" / "user")
        .and(warp::get())
        .and(routes::extract_auth_head())
        .and_then(handler_get_user)
}
pub async fn handler_get_user(auth: Auth) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_get_user: {:?}", auth);
    let conn = db::PG_POOL.clone().get().unwrap();
    let result = db::users::find(&conn, auth.id);
    match result {
        Some(user) => {
            let tmpjson = json!({ "user": user.to_user_auth() });
            Ok(warp::reply::with_status(
                warp::reply::json(&tmpjson),
                StatusCode::OK,
            ))
        }
        None => {
            let err = Errors::new(&[("get user ", "some error")]);
            err.respond_to()
        }
    }
}
// --------------------------------------------------------------------------------------------------------
#[derive(Deserialize, Debug)]
pub struct UpdateUser {
    user: db::users::UpdateUserData,
}

pub fn route_put_user() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path!("api" / "user")
        .and(warp::put())
        .and(warp::body::json())
        .and(routes::extract_auth_head())
        .and_then(handler_put_user)
}
pub async fn handler_put_user(
    user: UpdateUser,
    auth: Auth,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_put_user: {:?} \n {:?}", user, auth);
    let conn = db::PG_POOL.clone().get().unwrap();
    let result = db::users::update(&conn, auth.id, &user.user);
    match result {
        Some(user) => {
            let tmpjson = json!({ "user": user.to_user_auth() });
            Ok(warp::reply::with_status(
                warp::reply::json(&tmpjson),
                StatusCode::OK,
            ))
        }
        None => {
            let err = Errors::new(&[("put user ", "some error")]);
            err.respond_to()
        }
    }
}
