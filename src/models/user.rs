use crate::auth::Auth;
use chrono::{Duration, Utc};
use serde::Serialize;
use crate::config;
use crate::schema::users;

type Url = String;

#[table_name="users"]
#[derive(Queryable, Serialize,QueryableByName,Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
    pub image: Option<Url>,
    #[serde(skip_serializing)]
    pub hash: String,
}

#[derive(Queryable, Serialize)]
pub struct Follow {
    pub follower: i32,
    pub followed: i32,
}

#[derive(Serialize)]
pub struct UserAuth<'a> {
    username: &'a str,
    email: &'a str,
    bio: Option<&'a str>,
    image: Option<&'a str>,
    token: String,
}

#[derive(Serialize)]
pub struct Profile {
    username: String,
    bio: Option<String>,
    image: Option<String>,
    following: bool,
}

#[derive(Serialize)]
pub struct PubProfile {
    pub id: i32,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

impl User {
    pub fn to_user_auth(&self) -> UserAuth {
        let exp = Utc::now() + Duration::days(config::LOGIN_EXPIRE_DAYS); // TODO: move to config
        let token = Auth {
            id: self.id,
            username: self.username.clone(),
            exp: exp.timestamp(),
        }
        .token();

        UserAuth {
            username: &self.username,
            email: &self.email,
            bio: self.bio.as_ref().map(String::as_str),
            image: self.image.as_ref().map(String::as_str),
            token,
        }
    }

    pub fn to_profile(self, following: bool) -> Profile {
        Profile {
            username: self.username,
            bio: self.bio,
            image: self.image,
            following,
        }
    }
}
