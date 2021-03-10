// #[macro_use]
// extern crate diesel;

use diesel;
use diesel::pg::PgConnection;
use diesel::{prelude::*, sql_query};
// use serde::Serialize;
use structopt::StructOpt;

use realworld_warp::db;
use realworld_warp::schema::users;
//use realworld_warp::schema::follows;
use realworld_warp::models::user::User;
//use realworld_warp::models::user::Follow;
use realworld_warp::db::OffsetLimit;
use realworld_warp::db::articles::{ FindArticles, find };

#[derive(Debug, StructOpt)]
#[structopt(name = "realworld-cli", about = "An administrator tool for realworld blog system.")]
struct Opt {
    #[structopt(parse(from_str))]
    model: String,
    #[structopt(parse(from_str))]
    function: String,
    #[structopt(short, long)]
    all: bool,
    #[structopt(short, long)]
    email: Option<String>,
    #[structopt(short, long)]
    name: Option<String>,
    #[structopt(short, long)]
    title: Option<String>,
    #[structopt(short, long)]
    id: Option<i32>,
}
// in structopt, bool option is very interesting, only bool means FLAG giving is working, like "--all"
// if Option<bool>, then must "--all true|flase"
//----------------------------------------------------------------------
fn article_find(opt: &Opt, conn: &PgConnection){
    let au:Option<String>;
    let mut fa=FindArticles{
        tag: None,
        author: None,
        favorited: None,
        limit: Some(100),
        offset: Some(0),
    };
    match &opt.name {
        Some(a) => au = Some(a.clone()),
        None => {
            if opt.all{
                au = None;
            } else {
                println!("Need name provide, or list all...");
                return;
            }
        }
    };
    fa.author = au;
    let (res,_) = find(&conn,&fa,Some(0));
    for a in res{
        println!("\tid: {}\t title: {}",a.id, a.title);
    };
}
fn article_opt(opt: &Opt){
    let conn=db::PG_POOL.clone().get().unwrap();
    match opt.function.as_ref() {
        "ls" => article_find(&opt,&conn),
        // "delete" => user_delete(&opt, &conn),
        _ => println!("Error: no this function!")
    }
}

//----------------------------------------------------------------------
fn get_followers(uid: i32, conn: &PgConnection) { //-> Vec<User>{
    /*
    follows::table
        .left_join(
            users::table.on(users::id.eq(follows::follower))
        )
        .select((follows::all_columns,
            users::all_columns))
        .into_boxed()
        .filter(follows::followed.eq(uid))
        .offset_and_limit(0,100)
        .load_and_count::<( Follow, User)>(conn)
        .map(|(res,_)|{
            res.into_iter()
                .map(|(_, u)| u)
                .collect()
        }).ok().unwrap()
        */
    let res:Vec<User>=sql_query(
            format!("SELECT users.* FROM follows 
                LEFT JOIN users ON follows.follower=users.id 
                where follows.followed='{}'",uid)
        )
        .load(conn).unwrap();
    // println!("sql result: {:?}",res);
    for u in res{
        println!("\tid: {}\t name: {}",u.id, u.username);
    };
}

fn follow_list(opt: &Opt, conn: &PgConnection){
    get_users(&opt, &conn).map(|res|{
        for u in &res{
            println!("id: {}\t name: {}\t email: {}\t has Follower:",
                u.id, u.username, u.email);
            get_followers(u.id, &conn);
        }
    });
}
fn follow_opt(opt: &Opt){
    let conn=db::PG_POOL.clone().get().unwrap();
    match opt.function.as_ref() {
        "ls" => follow_list(&opt,&conn),
        // "delete" => user_delete(&opt, &conn),
        _ => println!("Error: no this function!")
    }
}

//----------------------------------------------------------------------

fn get_users(opt: &Opt, conn: &PgConnection) -> Option<Vec<User>>{
    let mut query = users::table
        .select(users::all_columns)
        .into_boxed();

    if let Some(mail)=&opt.email{
        let pattern = format!("%{}%", mail);
        query = query.filter(users::email.like(pattern))
    } else if let Some(name)=&opt.name {
        let pattern = format!("%{}%", name);
        query = query.filter(users::username.like(pattern))
    } else if let Some(uid)=&opt.id {
        query = query.filter(users::id.eq(uid))
    } else if opt.all {

    } else {
        println!("Error: [id | email | name | all] must provide...");
        return None;
    }
    query
        .offset_and_limit(0,100)
        .load_and_count::<User>(conn)
        .map(|(res1,_)|{
            Some(res1)
            // println!("{} record find.",count);
            // for u in &res1{
            //     println!("id:{} name:{}, email:{}",
            //         u.id, u.username, u.email);
            // }
        }).ok().unwrap()
        // .expect("Something wrong in db reading...");
}

fn user_delete(opt: &Opt, conn: &PgConnection){
    if let Some(uid)=&opt.id{
        diesel::delete(users::table.filter(users::id.eq(uid)))
        .execute(conn)
        .ok();
    } else {
        println!("Error: delete function must have <id> provide!");
        return;
    }
}
fn user_find(opt: &Opt, conn: &PgConnection){
    get_users(&opt, &conn).map(|res|{
        for u in &res{
            println!("id: {}\t name: {}\t email: {}",
                u.id, u.username, u.email);
        }
    });
    // println!("{}",u.len());
}

fn user_opt(opt: &Opt){
    let conn=db::PG_POOL.clone().get().unwrap();
    match opt.function.as_ref() {
        "ls" => user_find(&opt,&conn),
        "delete" => user_delete(&opt, &conn),
        _ => println!("Error: no this function!")
    }
}

//----------------------------------------------------------------------

fn main() {
    let opt = Opt::from_args();
    // println!("{:?}", opt);
    match opt.model.as_ref() {
        "user" => user_opt(&opt),
        "article" => article_opt(&opt),
        "follow" => follow_opt(&opt),
        _ => println!("Error: no this model!")
    }
}
