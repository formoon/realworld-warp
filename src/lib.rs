#[macro_use]
extern crate diesel;

use log::info;

mod auth;
mod config;
pub mod db;
mod errors;
pub mod models;
mod routes;
pub mod schema;

pub async fn launch() {
    let sn = db::CONFIG.server_name.to_string();
    info!("{} starting ...", sn);
    info!("Board Mode: {}", db::CONFIG.public_board.to_string());
    routes::web_routes().await;
}

// Run test code and output:
// cargo test -- --nocapture
#[cfg(test)]
mod tests {
    use crate::db;
    use crate::db::users;
    #[test]
    fn basic_db() {
        println!("{:?}", *db::CONFIG);
        let pgconn = db::PG_POOL.clone().get().unwrap();
        let ru = users::create(&pgconn, "admin", "andrewwang@sina.com", "12345678");
        match ru {
            Ok(_) => println!("create user ok!"),
            Err(_) => println!("user create error"),
        };
        let user = users::login(&pgconn, "andrewwang@sina.com", "12345678");
        println!("{}", user.as_ref().unwrap().hash);
        users::delete(&pgconn, user.unwrap().id);
    }
}
