use crate::prelude::*;
use std::collections::HashSet;
use crate::login::{LoginStage, COOKIE_SESSIONS_LIST, COOKIE_LAST_CHECK, COOKIE_LOGIN_STAGE, COOKIE_LOGIN_USER};
use actix_web::middleware::errhandlers::{ErrorHandlers, ErrorHandlerResponse};
use actix_web::{web, http, dev, App, HttpRequest, HttpResponse, Result};

#[derive(Debug, Serialize, Deserialize)]
struct DebugPageCtx {
    base: BasicCtx,
    url: String,
    version: String,
    method: String,
    ip_addr_real: String,
    ip_addr_peer: String,
    conn_type: String,
    headers: Vec<(String,String)>,
    session: Vec<(String,String)>,
}

#[get("/debug")]
async fn debug_get(data: web::Data<AppState>, session: Session, req: HttpRequest) -> impl Responder {
    debug_all(data, session, req).await
}

pub async fn debug_all(data: web::Data<AppState>, session: Session, req: HttpRequest) -> impl Responder {
    let mut headers = Vec::new();
    for header in req.headers().iter() {
        let name = header.0.to_string();
        let val = header.1.to_str();
        let val = match val {
            Ok(v) => v.to_string(),
            _ => format!("{:?}", val)
        };
        headers.push((name, val));
    }

    let mut smap = Vec::new();
    let key = COOKIE_SESSIONS_LIST.to_string();
    match session.get::<HashSet<Uuid>>(&key) {
        Ok(Some(v)) => smap.push((key, format!("{:?}", v))),
        Ok(None) => smap.push((key, "None".to_string())),
        Err(err) => smap.push((key, format!("ERR: {:?}", err))),
    };
    let key = COOKIE_LAST_CHECK.to_string();
    match session.get::<DateTime<Utc>>(&key) {
        Ok(Some(v)) => smap.push((key, format!("{:?}", v))),
        Ok(None) => smap.push((key, "None".to_string())),
        Err(err) => smap.push((key, format!("ERR: {:?}", err))),
    };
    let key = COOKIE_LOGIN_STAGE.to_string();
    match session.get::<LoginStage>(&key) {
        Ok(Some(v)) => smap.push((key, format!("{:?}", v))),
        Ok(None) => smap.push((key, "None".to_string())),
        Err(err) => smap.push((key, format!("ERR: {:?}", err))),
    };
    let key = COOKIE_LOGIN_USER.to_string();
    match session.get::<MinUser>(&key) {
        Ok(Some(v)) => smap.push((key, format!("{:?}", v))),
        Ok(None) => smap.push((key, "None".to_string())),
        Err(err) => smap.push((key, format!("ERR: {:?}", err))),
    };

    let (ip_addr_real, ip_addr_peer) = get_ip(&req);

    let base_ctx = BasicCtx::new("Debug".to_string(), None, true);
    let ctx = DebugPageCtx {
        base: base_ctx,
        method: req.method().to_string(),
        url: req.uri().to_string(),
        version: format!("{:?}", req.version()),
        conn_type: format!("{:?}", req.connection_info().clone()),
        ip_addr_real: ip_addr_real,
        ip_addr_peer: ip_addr_peer,
        headers: headers,
        session: smap,
    };
    exec_html_template(&data.tmpl, "debug.html", ctx)
}

#[get("/panic")]
#[allow(unreachable_code)]
async fn panic_get(_data: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
    panic!("Intentional panic to test 500 error page");
    exec_html_template(&_data.tmpl, "debug.html", BasicCtx::new("Error".to_string(), None, true))
}

#[get("/error")]
async fn error_get(_data: web::Data<AppState>, _req: HttpRequest) -> FResult<&'static str> {
    Err(FError::NotImplemented)
}

pub fn render_404<B>(res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    use actix_web::web::Bytes;
    use actix_web::dev::ServiceResponse;
    use actix_web::body::{Body, ResponseBody};

    let mut new_res: ServiceResponse<B> = res.map_body(|_head, _body| {
        let html = basic_html_template("error.html", EmptyCtx::new("Page Not Found".to_string(), None, true));
        ResponseBody::Other(Body::Bytes(Bytes::from(html)))
    });
    new_res.headers_mut().insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("text/html"));
    Ok(ErrorHandlerResponse::Response(new_res))
}

pub fn render_500<B>(res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    use actix_web::web::Bytes;
    use actix_web::dev::ServiceResponse;
    use actix_web::body::{Body, ResponseBody};

    let mut new_res: ServiceResponse<B> = res.map_body(|_head, _body| {
        let html = basic_html_template("error.html", EmptyCtx::new("Internal Server Error".to_string(), None, true));
        ResponseBody::Other(Body::Bytes(Bytes::from(html)))
    });
    new_res.headers_mut().insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("text/html"));
    Ok(ErrorHandlerResponse::Response(new_res))
}
