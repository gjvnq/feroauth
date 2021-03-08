mod login;
mod model;
mod prelude;
mod templates;

#[macro_use]
extern crate actix_web;
extern crate log;
extern crate serde_json;

use hex::FromHex;
use crate::prelude::*;

use actix_files as fs;
use actix_web::{App, HttpResponse, HttpServer};
use dotenv::dotenv;
use std::env;

#[get("/")]
async fn index(session: Session) -> Result<String, actix_web::Error> {
    // access session data
    if let Some(count) = session.get::<i32>("counter")? {
        println!("SESSION value: {}", count);
        session.set("counter", count + 1)?;
        Ok(format!("Welcome! {:?}", count))
    } else {
        session.set("counter", 1)?;
        Ok("Welcome!".to_string())
    }
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
    let cookie_key = <[u8; 32]>::from_hex(env::var("COOKIE_KEY").expect("COOKIE_KEY is not set in .env file")).expect("COOKIE_KEY must be exactly a 32 bytes hex encoded string");

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
            .wrap(
                CookieSession::signed(&cookie_key)
                    .name("feroauth")
                    .http_only(true)
                    .secure(false),
            )
            .service(fs::Files::new("/static", "static").prefer_utf8(true))
            .service(index)
            .service(login::login_get)
            .service(login::login_post)
    });

    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    server = server.bind(format!("{}:{}", host, port))?;

    info!("Starting server");
    server.run().await?;

    Ok(())
}
