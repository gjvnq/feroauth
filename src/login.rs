use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
enum LoginStage {
    AskUsername,
    AskPassword,
    AskSelect2FA,
    AskOTP,
    AskU2F,
}

#[derive(Debug)]
struct LoginFormInput {
    username: Option<String>,
    password: Option<String>,
    hashed_password: Option<String>,
    otp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginPageCtx {
    base: BasicCtx,
    username: String,
    hashed_password: String,
    stage: LoginStage,
}

#[get("/login")]
async fn login_get(data: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let query_str = req.query_string();
    let qs = QString::from(query_str);
    let username = qs.get("username").unwrap_or_default();

    let base_ctx = BasicCtx::new("Login".to_string(), None, true);
    let ctx = LoginPageCtx {
        base: base_ctx,
        username: username.to_string(),
        hashed_password: "".to_string(),
        stage: LoginStage::AskUsername,
    };
    exec_html_template(&data.tmpl, "login.html", ctx)
}

// #[post("/login", data = "<input>")]
// async fn login_post(input: Form<LoginFormInput>) -> Template {
//     let base_ctx = BasicCtx::new("Login".to_string(), None, true);
//     let username = input.username.clone().unwrap_or_default();
//     // get user
//     let mut tx = get_tx().await;
//     println!("{:?}", User::load_by_login_handle(&username, &mut tx).await);

//     let got_password = input.password.is_some() || input.hashed_password.is_some();
//     let stage = match input.username {
//         None => LoginStage::AskUsername,
//         Some(_) => match got_password {
//             false => LoginStage::AskPassword,
//             true => LoginStage::AskSelect2FA,
//         },
//     };
//     let hashed_password = match &input.hashed_password {
//         Some(v) => v.to_string(),
//         None => match &input.password {
//             Some(v) => hash_password(v.to_string()),
//             None => "".to_string(),
//         },
//     };
//     let ctx = LoginPageCtx {
//         base: base_ctx,
//         username: username,
//         hashed_password: hashed_password,
//         stage: stage,
//     };
//     Template::render("login", &ctx)
// }
