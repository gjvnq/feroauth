use crate::prelude::*;

#[get("/session/info")]
async fn get_session_info_endpoint(
    auth: FullSession,
) -> FResult<HttpResponse> {
    return Ok(HttpResponse::Ok().json(auth));
}