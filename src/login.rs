use crate::model::password::PasswordCheck;
use crate::prelude::*;
use std::collections::HashSet;

/// Indicates the cookie key that contains the list of active [`FSessionMin`].
const COOKIE_SESSIONS_LIST: &'static str = "sessions";
/// Indicates the cookie key that contains when the list of active sessions ([`COOKIE_SESSIONS_LIST`]) was last loaded from the DB.
const COOKIE_LAST_CHECK: &'static str = "last_check";
/// Current stage of the login process
const COOKIE_LOGIN_STAGE: &'static str = "login_stage";
/// The user that is currently trying to log in
const COOKIE_LOGIN_USER: &'static str = "login_user";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
enum LoginStage {
    AskUsername,
    AskPassword,
    AskSelect2FA,
    AskOTP,
    AskU2F,
    Done,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginFormInput {
    username: Option<String>,
    password: Option<String>,
    code_otp: Option<String>,
    code_u2f: Option<String>,
    selection_2fa: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginPageCtx {
    base: BasicCtx,
    username: String,
    password: String,
    stage: LoginStage,
    err_msg: String,
    user: Option<MinUser>
}

#[get("/login")]
async fn login_get(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    println!(
        "{:?}",
        crate::model::password::Password::load_by_user_uuid(
            Uuid::parse_str("d6fcb336-ee52-416d-9aa0-4a0f7d59612c").unwrap(),
            &mut get_tx().await,
        )
        .await
    );

    let query_str = req.query_string();
    let qs = QString::from(query_str);
    let username = qs.get("username").unwrap_or_default();

    let base_ctx = BasicCtx::new("Login".to_string(), None, true);
    let ctx = LoginPageCtx {
        base: base_ctx,
        username: username.to_string(),
        password: "".to_string(),
        err_msg: "".to_string(),
        stage: LoginStage::AskUsername,
        user: None,
    };
    exec_html_template(&data.tmpl, "login.html", ctx)
}

async fn login_ask_username(
    _data: &web::Data<AppState>,
    _input: &web::Form<LoginFormInput>,
    _session: &Session,
    _req: &HttpRequest,
    ctx: &mut LoginPageCtx
    ) -> bool {
    let mut tx = get_tx().await;
    if ctx.username.len() == 0 {
        ctx.stage = LoginStage::AskUsername;
        return true;
    }
    match MinUser::load_by_login_handle(&ctx.username, &mut tx).await {
        Err(err) => {
            if err.is_not_found() {
                warn!("User not found: {}", &ctx.username);
                ctx.err_msg = "User not found".to_string();
            } else {
                error!("Failed to find user for handle {}: {:?}", &ctx.username, err);
                ctx.err_msg = "Something went wrong. It's not your fault.".to_string();
            }
            return true
        }
        Ok(user) => {
            ctx.stage = LoginStage::AskPassword;
            ctx.user = Some(user);
            // This allows a script to send both the user and the password at the same time
            return false
        }
    }
}

async fn login_ask_password(
    _data: &web::Data<AppState>,
    input: &web::Form<LoginFormInput>,
    _session: &Session,
    _req: &HttpRequest,
    ctx: &mut LoginPageCtx
    ) -> bool {
    if ctx.user.is_none() {
        ctx.stage = LoginStage::AskUsername;
        return true;
    }
    let user = ctx.user.as_ref().unwrap();
    let password = match &input.password {
        Some(v) => v,
        None => "",
    };
    if password.len() > 0 {
        let mut tx = get_tx().await;
        match Password::verify_for_user(user.get_uuid(), &password, &mut tx)
            .await
        {
            Ok(PasswordCheck::WrongPassword) => {
                ctx.err_msg = "Wrong password.".to_string();
            }
            Ok(PasswordCheck::RightNo2FA) => {
                ctx.stage = LoginStage::Done;
            }
            Ok(PasswordCheck::RightNeeds2FA) => {
                ctx.stage = LoginStage::AskSelect2FA;
                return false;
            }
            Err(err) => {
                error!("{:?}", err);
                ctx.err_msg = "Something went wrong. It's not your fault.".to_string();
            }
        }
    }

    return true
}


#[post("/login")]
async fn login_post(
    data: web::Data<AppState>,
    input: web::Form<LoginFormInput>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {

    // Load necessary info
    let mut ctx = LoginPageCtx {
        base: BasicCtx::new("Login".to_string(), None, true),
        username: input.username.clone().unwrap_or_default(),
        password: input.password.clone().unwrap_or_default(),
        err_msg: "".to_string(),
        stage: LoginStage::AskUsername,
        user: None
    };
    ctx.stage = match session.get::<LoginStage>(COOKIE_LOGIN_STAGE) {
        Ok(Some(v)) => v,
        _ => LoginStage::AskUsername,
    };
    ctx.user = match session.get::<MinUser>(COOKIE_LOGIN_USER) {
        Ok(Some(v)) => Some(v),
        _ => None  
    };

    // Process what we recievd
    let mut ready = false;
    while !ready {
        ready = match ctx.stage {
            LoginStage::AskUsername => login_ask_username(&data, &input, &session, &req, &mut ctx).await,
            LoginStage::AskPassword => login_ask_password(&data, &input, &session, &req, &mut ctx).await,
            _ => unimplemented!()
        };
    };

    // If finished, create the FSession object
    if ctx.stage == LoginStage::Done {
        let user = ctx.user.as_ref().unwrap();
        // Safety: no sane client would fail to send the user-agent.
        let user_agent = req.headers().get("user-agent").unwrap().to_str().unwrap();
        // Safety: how on Earth could we answer a request without the client's IP address?
        let ip_addr = req.peer_addr().unwrap().ip();

        // Make and save fsession to record the login info
        let fsession = FSession::new(&user, &user, false, ip_addr, user_agent);
        let mut tx = get_tx().await;
        unwrap_or_log(fsession.save(&mut tx).await, "failed to save fsession");
        unwrap_or_log(tx.commit().await, "failed to save fsession");

        // Adjust cookies
        let mut session_set = match session.get::<HashSet<Uuid>>(COOKIE_SESSIONS_LIST) {
            Ok(Some(v)) => v,
            _ => HashSet::new(),
        };
        session_set.insert(fsession.get_uuid());
        unwrap_or_log(session.set(COOKIE_SESSIONS_LIST, session_set), "failed to set sessions list cookie");
        // Clear values we won't need
        session.remove(COOKIE_LOGIN_STAGE);
        session.remove(COOKIE_LOGIN_USER);
    } else {
        // Update cookies
        if let Err(err) = session.set(COOKIE_LOGIN_STAGE, ctx.stage) {
            error!("{:?}", err);
        }
        if let Some(user) = &ctx.user {
            if let Err(err) = session.set(COOKIE_LOGIN_USER, user) {
                error!("{:?}", err);
            }
        }
    }

    exec_html_template(&data.tmpl, "login.html", ctx)
}
