mod login;
mod model;
mod prelude;
mod templates;

#[macro_use]
extern crate actix_web;
extern crate log;
extern crate serde_json;

use crate::prelude::*;
use crate::login::login_get;

use actix_files as fs;
use actix_web::{App, HttpResponse, HttpServer};
use dotenv::dotenv;
use std::env;

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

#[actix_web::main]
async fn main() -> FResult<()> {
    dotenv().ok();
    env_logger::init();

    let db_host = env::var("DB_HOST").expect("DB_HOST is not set in .env file");
    let db_user = env::var("DB_USER").expect("DB_USER is not set in .env file");
    let db_pass = env::var("DB_PASS").expect("DB_PASS is not set in .env file");
    let db_name = env::var("DB_NAME").expect("DB_NAME is not set in .env file");
    let db_pool = model::db::get_pool(&db_host, &db_user, &db_pass, &db_name).await;

    unsafe {
        model::db::DB_POOL = Some(db_pool.clone());
    }

    let mut server = HttpServer::new(move || {
        App::new()
            .data(AppState {
                tmpl: templates::load_templates(),
                db: db_pool.clone(),
            })
            // add cookies
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
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
