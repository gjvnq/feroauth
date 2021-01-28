use crate::prelude::*;
use actix_web::HttpResponse;
use actix_web::Responder;

use tera::Context;
use tera::Tera;

pub fn load_templates() -> Tera {
    Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*.html"))
        .expect("Failed to load templates")
}

pub fn exec_html_template(
    tmpl: &Tera,
    tmpl_name: &str,
    ctx: impl Serialize,
) -> Either<impl Responder, &'static str> {
    let ctx = match Context::from_serialize(ctx) {
        Ok(v) => v,
        Err(err) => {
            error!("Failed to render template {}: {:?}", tmpl_name, err);
            return Either::B("Internal Server Error");
        }
    };
    let rendered = match tmpl.render(&tmpl_name, &ctx) {
        Ok(v) => v,
        Err(err) => {
            error!("Failed to render template {}: {:?}", tmpl_name, err);
            return Either::B("Internal Server Error");
        }
    };
    Either::A(HttpResponse::Ok().body(rendered))
}
