#![feature(proc_macro_hygiene, decl_macro)]

mod prelude;
use rocket::request::Form;
use crate::prelude::*;

#[macro_use] extern crate rocket;
extern crate serde_json;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::helmet::SpaceHelmet;
use rocket_contrib::templates::Template;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct BasicCtx {
    disable_vue: bool,
    page_title: String,
    page_desc: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginPageCtx {
    base: BasicCtx,
    username: String,
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
    otp: Option<String>
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/login?<username>")]
fn login1(username: Option<String>) -> Template {
    let base_ctx = BasicCtx { disable_vue: true, page_title: "abc".to_string(), page_desc: Some("dsds".to_string()) };
    let ctx = LoginPageCtx {
        base: base_ctx,
        username: username.unwrap_or_default(),
        stage: LoginStage::AskUsername
    };
    Template::render("login", &ctx)
}


#[post("/login", data = "<input>")]
fn login2(input: Form<LoginFormInput>) -> Template {
    let base_ctx = BasicCtx { disable_vue: true, page_title: "abc".to_string(), page_desc: Some("dsds".to_string()) };
    let ctx = LoginPageCtx {
        base: base_ctx,
        username: input.username.clone().unwrap_or_default(),
        stage: LoginStage::AskPassword
    };
    Template::render("login", &ctx)
}

fn main() {
    let helmet = SpaceHelmet::default();
    rocket::ignite()
        .attach(helmet)
        .attach(Template::fairing())
        .mount("/static", StaticFiles::from("static"))
        .mount("/", routes![index, login1, login2])
        .launch();
}