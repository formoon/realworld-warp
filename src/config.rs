/// js toISOString() in test suit can't handle chrono's default precision
pub const DATE_FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.3fZ";

pub const SECRET: &'static str = "secret123";
pub const TOKEN_PREFIX: &'static str = "Token ";
pub const LOGIN_EXPIRE_DAYS: i64 = 60;

pub const STATIC_PATH: &'static str = "./dist/";
pub const DOWNLOAD_PATH: &'static str = "./dl/";
pub const UPLOAD_DIR: &'static str = "./dl/";
