use crate::prelude::*;
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
    stage: LoginStage,
}

#[get("/login")]
async fn login_get(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    println!(
        "{:?}",
        crate::model::password::Password::load_by_user_uuid(
            Uuid::parse_str("d6fcb336-ee52-416d-9aa0-4a0f7d59612c").unwrap(),
            &mut data.db.begin().await.unwrap(),
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
    let mut user_not_found = false;
    let mut wrong_password = false;
    let mut internal_error = false;
    let mut tx = get_tx().await;
    let mut stage = match session.get::<LoginStage>("ferrocene_login_stage") {
        Ok(Some(v)) => v,
        _ => LoginStage::AskUsername,
    };
    let mut user = match session.get::<User>("ferrocene_login_user") {
        Ok(v) => v,
        _ => None,
    };

    // if user.is_some() {
    //     match User::load_by_uuid(&user_uuid, &mut tx).await {
    //         Ok(val) => user = val,
    //         Err(err) => {
    //             error!("Failed to load user {}: {:?}", user_uuid, err);
    //             user = None;
    //             stage = LoginStage::AskUsername;
    //         },
    //     }
    // }

    if user.is_none() {
        if input.username.is_some() {
            let username = input.username.as_ref().unwrap();
            match User::load_by_login_handle(username, &mut tx).await {
                Ok(val) => {
                    user = Some(val);
                }
                Err(err) => {
                    if err.is_not_found() {
                        warn!("User not found: {}", username);
                        user_not_found = true;
                    } else {
                        error!("Failed to find user for handle {}: {:?}", username, err);
                        internal_error = true;
                    }
                }
            }
            stage = LoginStage::AskPassword;
        } else {
            stage = LoginStage::AskUsername;
        }
    } else {
        if stage == LoginStage::AskPassword && user.is_some() {
            let password = input.password.clone().unwrap_or("".to_string());
            match Password::verify_for_user(user.as_ref().unwrap().get_uuid(), &password, &mut tx) {
                WrongPassword => {
                    wrong_password = true;
                },
                RightNo2FA => {
                    stage = LoginStage::Done;
                },
                RightNeeds2FA => {
                    stage = LoginStage::AskSelect2FA;
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
        // TODO: extend this for multiple logged in accounts
        if let Err(err) = session.set("ferrocene_current_user", user.unwrap().get_uuid()) {
            error!("{:?}", err);
        }
        // Clear values we won't need
        session.remove("ferrocene_login_stage");
        session.remove("ferrocene_login_user");
    } else {
        // Update cookies
        if let Err(err) = session.set("ferrocene_login_stage", stage) {
            error!("{:?}", err);
        }
        if let Err(err) = session.set("ferrocene_login_user", user) {
            error!("{:?}", err);
        }
    }

    let ctx = LoginPageCtx {
        base: BasicCtx::new("Login".to_string(), None, true),
        username: input.username.clone().unwrap_or_default(),
        stage: stage,
    };
    exec_html_template(&data.tmpl, "login.html", ctx)
}
