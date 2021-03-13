use crate::prelude::*;
use actix_web::HttpResponse;
use actix_web::Responder;

use tera::Context;
use tera::Tera;

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicCtx {
    pub disable_vue: bool,
    pub page_title: String,
    pub page_desc: Option<String>,
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
pub struct EmptyCtx {
    pub base: BasicCtx
}

impl EmptyCtx {
    pub fn new(title: String, desc: Option<String>, no_vue: bool) -> EmptyCtx {
        EmptyCtx {
            base: BasicCtx::new(title, desc, no_vue)
        }
    }
}

pub fn load_templates() -> Tera {
    Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*.html"))
        .expect("Failed to load templates")
}

pub fn basic_html_template(
    tmpl_name: &str,
    ctx: impl Serialize,
) -> String {
    let ctx = Context::from_serialize(ctx).map_err(|err| error!("Failed to render template {}: {:?}", tmpl_name, err)).unwrap();
    let rendered = get_tmpl().render(&tmpl_name, &ctx).map_err(|err| error!("Failed to render template {}: {:?}", tmpl_name, err)).unwrap();
    rendered
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
