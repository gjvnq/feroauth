use crate::model::password::PasswordCheck;
use crate::prelude::*;
use std::collections::HashSet;

/// Indicates the cookie key that contains the list of active [`FSessionMin`].
const COOKIE_SESSIONS_LIST: &'static str = "sessions";
/// Indicates the cookie key that contains when the list of active sessions ([`COOKIE_SESSIONS_LIST`]) was last loaded from the DB.
const COOKIE_LAST_CHECK: &'static str = "last_check";
const COOKIE_LOGIN_IN_STAGE: &'static str = "login_stage";
const COOKIE_LOGIN_IN_UUID: &'static str = "login_uuid";
const COOKIE_LOGIN_IN_NAME: &'static str = "login_name";

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
}

#[get("/login")]
async fn login_get(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    println!(
        "{:?}",
        crate::model::password::Password::load_by_user_uuid(
            Uuid::parse_str("d6fcb336-ee52-416d-9aa0-4a0f7d59612c").unwrap(),
            &mut data.new_tx().await.unwrap(),
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
    };
    exec_html_template(&data.tmpl, "login.html", ctx)
}

#[post("/login")]
async fn login_post(
    data: web::Data<AppState>,
    input: web::Form<LoginFormInput>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    println!("{:?}", req.headers());
    let user_agent = req.headers().get("user-agent").unwrap().to_str().unwrap();
    let ip_addr = req.peer_addr().unwrap().ip();
    println!("{}", user_agent);
    println!("{}", ip_addr);

    let mut tx = get_tx().await;
    let mut err_msg = "".to_string();
    let mut stage = match session.get::<LoginStage>(COOKIE_LOGIN_IN_STAGE) {
        Ok(Some(v)) => v,
        _ => LoginStage::AskUsername,
    };
    let mut user = None;
    let user_uuid = match session.get::<Uuid>(COOKIE_LOGIN_IN_UUID) {
        Ok(v) => v,
        _ => None,
    };

    if let Some(user_uuid) = user_uuid {
        match User::load_by_uuid(user_uuid, &mut tx).await {
            Ok(val) => {
                user = Some(val);
            }
            Err(err) => {
                if err.is_not_found() {
                    warn!("User not found: {}", user_uuid);
                    err_msg = "User not found".to_string();
                } else {
                    error!("Failed to find user for handle {}: {:?}", user_uuid, err);
                    err_msg = "Something went wrong. It's not your fault.".to_string();
                }
            }
        }
    } else {
        if input.username.is_some() {
            let username = input.username.as_ref().unwrap();
            match User::load_by_login_handle(username, &mut tx).await {
                Ok(val) => {
                    user = Some(val);
                }
                Err(err) => {
                    if err.is_not_found() {
                        warn!("User not found: {}", username);
                        err_msg = "User not found".to_string();
                    } else {
                        error!("Failed to find user for handle {}: {:?}", username, err);
                        err_msg = "Something went wrong. It's not your fault.".to_string();
                    }
                }
            }
            stage = LoginStage::AskPassword;
        } else {
            stage = LoginStage::AskUsername;
        }
    }

    if user.is_none() {
        stage = LoginStage::AskUsername;
    } else {
        let password = input.password.clone().unwrap_or("".to_string());
        if stage == LoginStage::AskPassword && user.is_some() && password.len() > 0 {
            debug!("Got password {:?}", input.password);
            match Password::verify_for_user(user.as_ref().unwrap().get_uuid(), &password, &mut tx)
                .await
            {
                Ok(PasswordCheck::WrongPassword) => {
                    err_msg = "Wrong password.".to_string();
                }
                Ok(PasswordCheck::RightNo2FA) => {
                    stage = LoginStage::Done;
                }
                Ok(PasswordCheck::RightNeeds2FA) => {
                    stage = LoginStage::AskSelect2FA;
                }
                Err(err) => {
                    error!("{:?}", err);
                    err_msg = "Something went wrong. It's not your fault.".to_string();
                }
            }
        }

        if stage == LoginStage::AskSelect2FA {
            // todo
            let _password = input.selection_2fa.clone().unwrap_or("".to_string());
        }

        if stage == LoginStage::AskOTP {
            // todo
            let _code_otp = input.code_otp.clone().unwrap_or("".to_string());
        }

        if stage == LoginStage::AskU2F {
            // todo
            let _code_u2f = input.code_u2f.clone().unwrap_or("".to_string());
        }
    }

    if stage == LoginStage::Done && user.is_some() {
        let min_user = user.as_ref().unwrap().as_min_user();
        let fsession = FSession::new(&min_user, &min_user, false, ip_addr, user_agent);
        let mut tx = data.new_tx().await.unwrap();
        println!("{:?}", fsession.save(&mut tx).await);
        println!("{:?}", tx.commit().await);
        // TODO: extend this for multiple logged in accounts
        let mut session_set = match session.get::<HashSet<Uuid>>(COOKIE_SESSIONS_LIST) {
            Ok(Some(v)) => v,
            _ => HashSet::new(),
        };
        session_set.insert(fsession.get_uuid());
        if let Err(err) = session.set(COOKIE_SESSIONS_LIST, session_set) {
            error!("{:?}", err);
        }
        // Clear values we won't need
        session.remove(COOKIE_LOGIN_IN_STAGE);
        session.remove(COOKIE_LOGIN_IN_UUID);
        session.remove(COOKIE_LOGIN_IN_NAME);
    } else {
        // Update cookies
        if let Err(err) = session.set(COOKIE_LOGIN_IN_STAGE, stage) {
            error!("{:?}", err);
        }
        if let Err(err) = session.set(COOKIE_LOGIN_IN_UUID, user_uuid) {
            error!("{:?}", err);
        }
        if let Some(user) = user {
            if let Err(err) = session.set(COOKIE_LOGIN_IN_NAME, user.display_name) {
                error!("{:?}", err);
            }
        }
    }

    let ctx = LoginPageCtx {
        base: BasicCtx::new("Login".to_string(), None, true),
        username: input.username.clone().unwrap_or_default(),
        password: input.password.clone().unwrap_or_default(),
        err_msg: err_msg,
        stage: stage,
    };
    exec_html_template(&data.tmpl, "login.html", ctx)
}
