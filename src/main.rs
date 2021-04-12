mod auth;
mod misc;
mod model;
mod prelude;
mod users;

#[macro_use]
extern crate actix_web;
extern crate log;
extern crate serde_json;

use crate::prelude::*;

use actix_web::{App, HttpServer};
use actix_web::http::header;
use actix_cors::Cors;
use dotenv::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> FResult<()> {
    dotenv().ok();
    env_logger::init();

    let db_host = env::var("DB_HOST").expect("DB_HOST is not set in .env file");
    let db_user = env::var("DB_USER").expect("DB_USER is not set in .env file");
    let db_pass = env::var("DB_PASS").expect("DB_PASS is not set in .env file");
    let db_name = env::var("DB_NAME").expect("DB_NAME is not set in .env file");
    let db_pool = model::db::get_pool(&db_host, &db_user, &db_pass, &db_name).await;
    let origin = env::var("ORIGIN").expect("ORIGIN is not set in .env file");
    info!("Allowing ORIGIN: {}", origin);

    let mut enforcer = PolicyEnforcer::new()?;
    let mut tx = db_pool.begin().await?;
    enforcer.reload(&mut tx).await?;
    drop(tx);

    let mut server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&origin)
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE, header::COOKIE])
            .allow_any_method()
            .max_age(30);

        App::new()
            .wrap(cors)
            .data(AppState {
                db: db_pool.clone(),
                enforcer: enforcer.clone()
            })
            .wrap(crate::auth::SessionAuth::new("feroauth", db_pool.clone()))
            .service(auth::validate_endpoint)
            .service(users::login_endpoint)
            .service(users::get_user_endpoint)
            .service(users::put_user_endpoint)
            .service(misc::get_session_info_endpoint)
    });

    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    server = server.bind(format!("{}:{}", host, port))?;

    info!("Starting server on {}:{}", host, port);
    server.run().await?;

    Ok(())
}
