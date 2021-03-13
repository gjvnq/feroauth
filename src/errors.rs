use crate::prelude::*;
use actix_web::http::HeaderValue;
use actix_web::middleware::errhandlers::ErrorHandlerResponse;

use actix_web::http::header;

use actix_web::{HttpRequest, Responder};
use tera::Context;

use actix_web::body::{Body, ResponseBody};
use actix_web::dev::ServiceResponse;
use actix_web::web::Bytes;

fn safe_html_template(tmpl_name: &str, raw_ctx: impl Serialize + std::fmt::Debug) -> String {
    let ctx = match Context::from_serialize(&raw_ctx) {
        Ok(v) => v,
        Err(err) => {
            error!(
                "Failed to serialize: {:?} (on template {})- {:?}",
                &raw_ctx, tmpl_name, err
            );
            return "Internal Server Error".to_string();
        }
    };
    let rendered = match get_tmpl().render(&tmpl_name, &ctx) {
        Ok(v) => v,
        Err(err) => {
            error!("Failed to render template {}: {:?}", tmpl_name, err);
            return "Internal Server Error".to_string();
        }
    };
    rendered
}

#[get("/panic")]
#[allow(unreachable_code)]
async fn panic_get(_req: HttpRequest) -> impl Responder {
    panic!("Intentional panic to test 500 error page");
    exec_html_template("debug.html", BasicCtx::new("Error".to_string(), None, true))
}

#[get("/error")]
async fn error_get(_data: web::Data<AppState>, _req: HttpRequest) -> FResult<&'static str> {
    Err(FError::NotImplemented)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorCtx {
    pub base: BasicCtx,
    pub code: i32,
}

impl ErrorCtx {
    pub fn new(title: String, desc: Option<String>, code: i32) -> ErrorCtx {
        ErrorCtx {
            base: BasicCtx::new(title, desc, false),
            code: code,
        }
    }
}

pub fn render_404<B>(res: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<B>> {
    let mut new_res: ServiceResponse<B> = res.map_body(|_head, _body| {
        let html = safe_html_template(
            "error.html",
            ErrorCtx::new("Page Not Found".to_string(), None, 404),
        );
        ResponseBody::Other(Body::Bytes(Bytes::from(html)))
    });
    new_res
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
    Ok(ErrorHandlerResponse::Response(new_res))
}

pub fn render_500<B>(res: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<B>> {
    let mut new_res: ServiceResponse<B> = res.map_body(|_head, _body| {
        let html = safe_html_template(
            "error.html",
            ErrorCtx::new("Internal Server Error".to_string(), None, 500),
        );
        ResponseBody::Other(Body::Bytes(Bytes::from(html)))
    });
    new_res
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
    Ok(ErrorHandlerResponse::Response(new_res))
}
