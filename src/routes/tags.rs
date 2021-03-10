use crate::db;
use std::convert::Infallible;
use warp::Filter;
use serde_json::json;

pub fn route_get_tags() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("api" / "tags"))
        .and_then(handler_get_tags)
}
pub async fn handler_get_tags() -> Result<impl warp::Reply, Infallible> {
    log::debug!("handler_get_tags...");
    let conn=db::PG_POOL.clone().get().unwrap();
    let tmpjson=json!({ "tags": db::articles::tags(&conn)});
    Ok(warp::reply::json(&tmpjson))
}
