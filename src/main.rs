mod auth;
mod config;
mod db;
mod prelude;
mod user;
mod templates;

#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate log;

extern crate serde_json;

use tera::Context;
use actix_web::HttpRequest;
use tera::Tera;
use tera::Template;
use sqlx::mysql::MySqlPoolOptions;
use dotenv::dotenv;

use crate::prelude::*;

use std::env;


use actix_files as fs;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::prelude::ConnectOptions;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};

struct AppState {
    tmpl: Tera,
    db: sqlx::Pool<sqlx::MySql>
}


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

#[derive(Debug)]
struct LoginFormInput {
    username: Option<String>,
    password: Option<String>,
    hashed_password: Option<String>,
    otp: Option<String>,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(r#"
        Welcome to Actix-web with SQLx Todos example.
        Available routes:
        GET /todos -> list of all todos
        POST /todo -> create new todo, example: { "description": "learn actix and sqlx", "done": false }
        GET /todo/{id} -> show one todo with requested id
        PUT /todo/{id} -> update todo with requested id, example: { "description": "learn actix and sqlx", "done": true }
        DELETE /todo/{id} -> delete todo with requested id
    "#
    )
}

#[get("/login")]
async fn login_get(data: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
    let base_ctx = BasicCtx::new("Login".to_string(), None, true);
    let ctx = LoginPageCtx {
        base: base_ctx,
        username: "".to_string(),
        hashed_password: "".to_string(),
        stage: LoginStage::AskUsername,
    };
    // firgure out whatever the hell rocket-contrib did to make this kind of code more pleasant
    // let mut ctx = Context::new();
    // ctx.insert("name", "hi");
    // let rendered = data.tmpl.render("login.html", &ctx).unwrap();
    // HttpResponse::Ok().body(rendered)
    exec_html_template(&data.tmpl, "login.html", ctx)
}

// #[get("/login?<username>")]
// fn login_get(username: Option<String>) -> Template {
//     let base_ctx = BasicCtx::new("Login".to_string(), None, true);
//     let ctx = LoginPageCtx {
//         base: base_ctx,
//         username: username.unwrap_or_default(),
//         hashed_password: "".to_string(),
//         stage: LoginStage::AskUsername,
//     };
//     Template::render("login", &ctx)
// }

// #[post("/login", data = "<input>")]
// async fn login_post(input: Form<LoginFormInput>) -> Template {
//     let base_ctx = BasicCtx::new("Login".to_string(), None, true);
//     let username = input.username.clone().unwrap_or_default();
//     // get user
//     let mut tx = get_tx().await;
//     println!("{:?}", User::load_by_login_handle(&username, &mut tx).await);

//     let got_password = input.password.is_some() || input.hashed_password.is_some();
//     let stage = match input.username {
//         None => LoginStage::AskUsername,
//         Some(_) => match got_password {
//             false => LoginStage::AskPassword,
//             true => LoginStage::AskSelect2FA,
//         },
//     };
//     let hashed_password = match &input.hashed_password {
//         Some(v) => v.to_string(),
//         None => match &input.password {
//             Some(v) => hash_password(v.to_string()),
//             None => "".to_string(),
//         },
//     };
//     let ctx = LoginPageCtx {
//         base: base_ctx,
//         username: username,
//         hashed_password: hashed_password,
//         stage: stage,
//     };
//     Template::render("login", &ctx)
// }

#[actix_web::main]
async fn main() -> FResult<()> {
    dotenv().ok();
    env_logger::init();

    let db_host = env::var("DB_HOST").expect("DB_HOST is not set in .env file");
    let db_user = env::var("DB_USER").expect("DB_USER is not set in .env file");
    let db_pass = env::var("DB_PASS").expect("DB_PASS is not set in .env file");
    let db_name = env::var("DB_NAME").expect("DB_NAME is not set in .env file");
    let mut conn_opts = MySqlConnectOptions::new()
        .host(&db_host)
        .username(&db_user)
        .password(&db_pass)
        .database(&db_name);
    conn_opts.log_statements(log::LevelFilter::Debug);
    let db_pool = MySqlPoolOptions::new()
        .connect_with(conn_opts).await?;

    unsafe {
        db::DB_POOL = Some(db_pool.clone());
    }

    let mut server = HttpServer::new(move || {
        App::new()
            .data(AppState{
                tmpl: templates::load_templates(),
                db: db_pool.clone()
            })
            .service(fs::Files::new("/static", "static").prefer_utf8(true))
            .service(index)
            .service(login_get)
    });

    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    server = server.bind(format!("{}:{}", host, port))?;

    info!("Starting server");
    server.run().await?;

    Ok(())
}
