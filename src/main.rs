#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::helmet::SpaceHelmet;
use rocket_contrib::templates::Template;

#[derive(Serialize)]
struct BasicCtx {
    disable_vue: bool,
    page_title: String,
    page_desc: Option<String>
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/login")]
fn login() -> Template {
    let context = BasicCtx { disable_vue: true, page_title: "abc".to_string(), page_desc: Some("dsds".to_string()) };
    Template::render("login", &context)
}

fn main() {
    let helmet = SpaceHelmet::default();
    rocket::ignite()
        .attach(helmet)
        .attach(Template::fairing())
        .mount("/static", StaticFiles::from("static"))
        .mount("/", routes![index, login])
        .launch();
}