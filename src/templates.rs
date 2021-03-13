use crate::prelude::*;
use actix_web::HttpResponse;

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
    pub base: BasicCtx,
}

impl EmptyCtx {
    #[allow(unused)]
    pub fn new(title: String, desc: Option<String>, no_vue: bool) -> EmptyCtx {
        EmptyCtx {
            base: BasicCtx::new(title, desc, no_vue),
        }
    }
}

pub fn load_templates() -> Tera {
    Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*.html"))
        .expect("Failed to load templates")
}

pub fn exec_html_template(
    tmpl_name: &str,
    raw_ctx: impl Serialize + std::fmt::Debug,
) -> FResult<HttpResponse> {
    let ctx = match Context::from_serialize(&raw_ctx) {
        Ok(v) => v,
        Err(err) => {
            error!(
                "Failed to serialize: {:?} (on template {})- {:?}",
                &raw_ctx, tmpl_name, err
            );
            return Err(FError::SerializationError(err.to_string()));
        }
    };
    let rendered = match get_tmpl().render(&tmpl_name, &ctx) {
        Ok(v) => v,
        Err(err) => {
            error!("Failed to render template {}: {:?}", tmpl_name, err);
            return Err(FError::TemplateError(
                tmpl_name.to_string(),
                err.to_string(),
            ));
        }
    };
    let mut ans = HttpResponse::Ok().body(rendered);
    ans.headers_mut().insert(
        actix_web::http::header::CONTENT_TYPE,
        actix_web::http::HeaderValue::from_static("text/html"),
    );
    Ok(ans)
}
