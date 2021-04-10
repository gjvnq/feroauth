use crate::model::password::PasswordCheck;
use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize, Default)]
struct LoginRequest {
    username: String,
    password: String,
    code_otp: String,
    code_u2f: String,
    selection_2fa: String,
    remember_me: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum LoginResponseStatus {
    MissingUsername,
    UserNotFound,
    MissingPassword,
    WrongPassword,
    Select2FA,
    Wrong2FA,
    LoggedIn,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    status: LoginResponseStatus,
    user: Option<MinUser>,
}

impl LoginResponse {
    fn new(status: LoginResponseStatus) -> LoginResponse {
        LoginResponse {
            status: status,
            user: None,
        }
    }
}

#[post("/login")]
async fn login_endpoint(
    data: web::Data<AppState>,
    info: web::Json<LoginRequest>,
    mut req: HttpRequest,
) -> FResult<HttpResponse> {
    // Safety: no sane client would fail to send the user-agent.
    let user_agent = req.headers().get("user-agent").unwrap().to_str().unwrap();

    let time_start = Utc::now().timestamp_millis();
    debug!(
        "{} - Start login for {:?}",
        Utc::now().timestamp_millis() - time_start,
        info
    );
    if info.username.len() == 0 {
        return Ok(
            HttpResponse::Ok().json(LoginResponse::new(LoginResponseStatus::MissingUsername))
        )
    }
    debug!(
        "{} - Got username",
        Utc::now().timestamp_millis() - time_start
    );

    let mut tx = data.db.begin().await.unwrap();
    let user = match MinUser::load_by_login_handle(&info.username, &mut tx).await {
        Ok(v) => v,
        Err(err) => {
            debug!("{:?}", err);
            if err.is_not_found() {
                return Ok(
                    HttpResponse::Ok().json(LoginResponse::new(LoginResponseStatus::UserNotFound))
                );
            } else {
                return Err(err);
            }
        }
    };
    debug!("{} - Got user", Utc::now().timestamp_millis() - time_start);

    let mut ans = LoginResponse {
        status: LoginResponseStatus::MissingPassword,
        user: Some(user.clone()),
    };
    if info.password.len() == 0 {
            ans.status = LoginResponseStatus::MissingPassword;
            return Ok(HttpResponse::Ok().json(ans));
    }
    debug!(
        "{} - Got password",
        Utc::now().timestamp_millis() - time_start
    );
    match Password::verify_for_user(user.get_uuid(), &info.password, &mut tx).await? {
        PasswordCheck::WrongPassword => {
            debug!(
                "{} - Finished login for {:?}",
                Utc::now().timestamp_millis() - time_start,
                info
            );
            ans.status = LoginResponseStatus::WrongPassword;
            return Ok(HttpResponse::Ok().json(ans));
        }
        PasswordCheck::RightNeeds2FA => ans.status = LoginResponseStatus::Select2FA,
        PasswordCheck::RightNo2FA => ans.status = LoginResponseStatus::LoggedIn,
    };
    debug!(
        "{} - Verified password",
        Utc::now().timestamp_millis() - time_start
    );

    if ans.status == LoginResponseStatus::Select2FA {
        unimplemented!()
    }

    if ans.status == LoginResponseStatus::LoggedIn {
        let (ip_addr_real, ip_addr_peer) = get_ip(&req);
        let remember_me = info.remember_me;
        let user = User::load_by_uuid(user.get_uuid(), &User::system_super_user(), &data.enforcer, &mut tx).await?;
        let session = FullSession::new(
            &user,
            &user,
            remember_me,
            &ip_addr_real,
            &ip_addr_peer,
            user_agent,
        );
        session.save(&mut tx).await?;
        tx.commit().await?;

        session.to_request(&mut req);

        debug!(
            "{} - Finished login for {:?}",
            Utc::now().timestamp_nanos() - time_start,
            info
        );
        return Ok(HttpResponse::Ok().json(ans));
    } else {
        unreachable!()
    }
}

#[get("/users/{handle}")]
async fn get_user_endpoint(
    data: web::Data<AppState>,
    auth: FullSession,
    path: web::Path<String>,
) -> FResult<HttpResponse> {
    // TODO: check permission

    let mut tx = data.db.begin().await.unwrap();
    let user = User::load_by_login_handle(&path, auth.get_user(), &data.enforcer, &mut tx).await?;

    return Ok(HttpResponse::Ok().json(user));
}

#[put("/users/{handle}")]
async fn put_user_endpoint(
    data: web::Data<AppState>,
    auth: FullSession,
    info: web::Json<UserChange>,
    path: web::Path<String>,
) -> FResult<HttpResponse> {
    // TODO: check permission

    let mut tx = data.db.begin().await.unwrap();
    let handle = path.as_str();
    let mut user = match handle {
        "new" => User::new(),
        _ => User::load_by_login_handle(handle, auth.get_user(), &data.enforcer, &mut tx).await?,
    };
    user.apply_changes(info.into_inner());
    debug!("{:?}", user);
    user.save(auth.get_user(), &data.enforcer, &mut tx).await?;
    tx.commit().await?;

    return Ok(HttpResponse::Ok().json(user));
}
