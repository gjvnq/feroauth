use crate::login::{
    LoginStage, COOKIE_LAST_CHECK, COOKIE_LOGIN_STAGE, COOKIE_LOGIN_USER, COOKIE_SESSIONS_LIST,
};
use crate::prelude::*;
use actix_web::HttpRequest;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
struct DebugPageCtx {
    base: BasicCtx,
    url: String,
    version: String,
    method: String,
    ip_addr_real: String,
    ip_addr_peer: String,
    conn_type: String,
    headers: Vec<(String, String)>,
    session: Vec<(String, String)>,
}

#[get("/debug")]
async fn debug_get(session: Session, req: HttpRequest) -> impl Responder {
    debug_all(session, req).await
}

pub async fn debug_all(session: Session, req: HttpRequest) -> impl Responder {
    let mut headers = Vec::new();
    for header in req.headers().iter() {
        let name = header.0.to_string();
        let val = header.1.to_str();
        let val = match val {
            Ok(v) => v.to_string(),
            _ => format!("{:?}", val),
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
    exec_html_template("debug.html", ctx)
}
