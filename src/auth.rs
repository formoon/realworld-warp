use serde::{Deserialize, Serialize};
use serde_json::json; //not use json! in rocket_contrib
use frank_jwt::{Algorithm, encode, decode, ValidationOptions};
use crate::db;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Auth {
    /// timestamp
    pub exp: i64,
    /// user id
    pub id: i32,
    pub username: String,
}
impl Auth {
    pub fn token(&self) -> String {
        let headers = json!({});
        let payload = json!(self);
        let jwt=encode(
            headers,
            &db::CONFIG.secret_key,
            // &config::SECRET.to_string(),
            &payload,
            Algorithm::HS256,
        );
        jwt.unwrap()
    }
}

/// Decode token into `Auth` struct. If any error is encountered, log it
/// an return None.
pub fn decode_token(token: &str) -> Option<Auth> {
    decode(token, 
        // &config::SECRET.to_string(), 
        &db::CONFIG.secret_key,
        Algorithm::HS256, &ValidationOptions::default())
        .map(|(_, payload)| {
            serde_json::from_value::<Auth>(payload)
                .map_err(|err| {
                    eprintln!("Auth serde decode error: {:?}", err);
                })
                .ok()
        })
        .unwrap_or_else(|err| {
            eprintln!("Auth decode error: {:?}", err);
            None
        })
}
