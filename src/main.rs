#![feature(proc_macro_hygiene, decl_macro)]

mod prelude;
// mod user;
mod config;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
// use crate::prelude::*;
extern crate serde_json;

use rocket::request::Form;
use serde::{Deserialize, Serialize};

use rocket_contrib::helmet::SpaceHelmet;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use sqlx::mysql::MySqlPoolOptions;

#[derive(Debug, Serialize, Deserialize)]
struct BasicCtx {
    disable_vue: bool,
    page_title: String,
    page_desc: Option<String>,
}

impl BasicCtx {
    pub fn new(title: String, desc: Option<String>, no_vue: bool) -> BasicCtx {
        BasicCtx {
            disable_vue: no_vue,
            page_desc: desc,
            page_title: title,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginPageCtx {
    base: BasicCtx,
    username: String,
    hashed_password: String,
    stage: LoginStage,
}

#[derive(Debug, Serialize, Deserialize)]
enum LoginStage {
    AskUsername,
    AskPassword,
    AskSelect2FA,
    AskOTP,
    AskU2F,
}

#[derive(Debug, FromForm)]
struct LoginFormInput {
    username: Option<String>,
    password: Option<String>,
    hashed_password: Option<String>,
    otp: Option<String>,
}

#[get("/")]
fn index() -> &'static str {
    // use crate::schema::user::dsl::*;
    // println!("{:?}", user.load::<crate::user::UserRaw>(&conn.0));
    "Hello, world!"
}

#[get("/login?<username>")]
fn login_get(username: Option<String>) -> Template {
    let base_ctx = BasicCtx::new("Login".to_string(), None, true);
    let ctx = LoginPageCtx {
        base: base_ctx,
        username: username.unwrap_or_default(),
        hashed_password: "".to_string(),
        stage: LoginStage::AskUsername,
    };
    Template::render("login", &ctx)
}

fn hash_password(_pass: String) -> String {
    // TODO: hash SHA-256
    return "".to_string();
}

#[post("/login", data = "<input>")]
fn login_post(input: Form<LoginFormInput>) -> Template {
    let base_ctx = BasicCtx::new("Login".to_string(), None, true);
    let username = input.username.clone().unwrap_or_default();
    // get user

    let got_password = input.password.is_some() || input.hashed_password.is_some();
    let stage = match input.username {
        None => LoginStage::AskUsername,
        Some(_) => match got_password {
            false => LoginStage::AskPassword,
            true => LoginStage::AskSelect2FA,
        },
    };
    let hashed_password = match &input.hashed_password {
        Some(v) => v.to_string(),
        None => match &input.password {
            Some(v) => hash_password(v.to_string()),
            None => "".to_string(),
        },
    };
    let ctx = LoginPageCtx {
        base: base_ctx,
        username: username,
        hashed_password: hashed_password,
        stage: stage,
    };
    Template::render("login", &ctx)
}

#[tokio::main]
async fn main() {
    let config = config::load_config();
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&config.db)
        .await
        .unwrap();
    println!("{:?}", pool);
    // Make a simple query to return the given parameter
    let row: Result<(i64,), _> = sqlx::query_as("SELECT ?")
        .bind(150_i64)
        .fetch_one(&pool)
        .await;
    println!("{:?}", row);

    let helmet = SpaceHelmet::default();
    rocket::ignite()
        .attach(helmet)
        .attach(Template::fairing())
        .mount("/static", StaticFiles::from("static"))
        .mount("/", routes![index, login_get, login_post])
        .launch();
}
