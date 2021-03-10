use std::env;
use diesel::PgConnection;
use r2d2_diesel::ConnectionManager;
use r2d2;
use lazy_static::lazy_static;
use dotenv::dotenv;
use pretty_env_logger;
use std::sync::Mutex;
use crate::config;
// use std::str::FromStr;

pub mod articles;
pub mod comments;
pub mod profiles;
pub mod users;

#[derive(Debug)]
pub struct Config {
    pub server_name: String,
    pub web_url: String,
    // pub port: u16,
    pub database_url: String,
    pub secret_key: String,
    pub public_board: bool,
    // pub log_level: String,
}
impl Config {
    fn read_config() -> Config {
        dotenv().ok();
        pretty_env_logger::init();

        let web_url = env::var("WEB_URL")
            .unwrap_or_else(|_| "localhost:8000".to_string());
        let server_name = env::var("SERVER_NAME")
            .unwrap_or_else(|_| "Mute Server".to_string());
        let public_board = env::var("PUBLIC_BOARD")
            .unwrap_or_else(|_| String::from("false")) == "true";
          
        let secret_key = env::var("SECRET_KEY")
            .unwrap_or_else(|_| config::SECRET.to_string().clone());
    
        let database_url =
            env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found");

            Config{
            server_name,
            web_url,
            database_url,
            secret_key,
            public_board,
            // log_level,
        }
    }
}

fn pg_poll(url: &String,pool_size: u32) -> r2d2::Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(url);

    r2d2::Pool::builder().max_size(pool_size).build(manager)
        .unwrap()
}
lazy_static! {
    pub static ref CONFIG:Config = Config::read_config();
    pub static ref PG_POOL:r2d2::Pool<ConnectionManager<PgConnection>>
         = pg_poll(&CONFIG.database_url, 10);
    pub static ref UPLOAD_LIST:Mutex<Vec<String>> = Mutex::new(Vec::new());
}

// #[database("diesel_postgres_pool")]
// pub struct Conn(diesel::PgConnection);

use diesel::prelude::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::query_builder::*;
use diesel::pg::Pg;
use diesel::sql_types::BigInt;

pub trait OffsetLimit: Sized {
    fn offset_and_limit(self, offset: i64, limit: i64) -> OffsetLimited<Self>;
}

impl<T> OffsetLimit for T {
    fn offset_and_limit(self, offset: i64, limit: i64) -> OffsetLimited<Self> {
        OffsetLimited {
            query: self,
            limit,
            offset,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct OffsetLimited<T> {
    query: T,
    offset: i64,
    limit: i64,
}

impl<T> OffsetLimited<T> {

    pub fn load_and_count<U>(self, conn: &PgConnection) -> QueryResult<(Vec<U>, i64)>
    where
        Self: LoadQuery<PgConnection, (U, i64)>,
    {
        let results = self.load::<(U, i64)>(conn)?;
        let total = results.get(0).map(|x| x.1).unwrap_or(0);
        let records = results.into_iter().map(|x| x.0).collect();
        Ok((records, total))
    }
}

impl<T: Query> Query for OffsetLimited<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<PgConnection> for OffsetLimited<T> {}

impl<T> QueryFragment<Pg> for OffsetLimited<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.limit)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.offset)?;
        Ok(())
    }
}