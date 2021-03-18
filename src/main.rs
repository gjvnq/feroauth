mod misc;
mod users;
mod model;
mod prelude;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate actix_web;
extern crate log;
extern crate serde_json;

use crate::prelude::*;
use hex::FromHex;

use actix_web::{http, App, HttpServer};
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
    let cookie_key =
        <[u8; 32]>::from_hex(env::var("COOKIE_KEY").expect("COOKIE_KEY is not set in .env file"))
            .expect("COOKIE_KEY must be exactly a 32 bytes hex encoded string");

    unsafe {
        model::db::DB_POOL = Some(db_pool.clone());
    }

    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes = ring::signature::EcdsaKeyPair::generate_pkcs8(&ring::signature::ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();

    let enc_key = jsonwebtoken::EncodingKey::from_ec_der(pkcs8_bytes.as_ref());
    let dec_key = jsonwebtoken::DecodingKey::from_ec_der(pkcs8_bytes.as_ref()).into_static();
    debug!("JWT key: {}", base64::encode(pkcs8_bytes.as_ref()));
    info!("Public  JWT key: {:?}", dec_key);

    set_jwt_config(JwtConfig{
        alg: Some(JwtAlgorithm::ES256),
        kid: "".to_string(),
        jku: "".to_string(),
        enc_key: Some(enc_key),
        dec_key: Some(dec_key)
    })?;

    let p = Password::new(Uuid::new_v4(), "admin", false);
    println!("{:?}", p);
    let t = Utc::now().timestamp_millis();
    println!("{:?}", p.unwrap().just_verify("admin"));
    println!("{}", Utc::now().timestamp_millis()-t);

    let mut server = HttpServer::new(move || {
        App::new()
            .data(AppState {
                db: db_pool.clone(),
            })
            // add cookies
            .wrap(
                CookieSession::signed(&cookie_key)
                    .name("feroauth")
                    .http_only(true)
                    .secure(false),
            )
            .service(index)
            .service(users::login)
    });

    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    server = server.bind(format!("{}:{}", host, port))?;

    info!("Starting server on {}:{}", host, port);
    server.run().await?;

    Ok(())
}
