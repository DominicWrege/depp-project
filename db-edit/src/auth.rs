use crate::config::hash;
use crate::error::HttpError;
use crate::handler::{redirect, render_template, HttpResult};
use crate::template::TEMPLATES;
use crate::State;
use actix_identity::Identity;
use actix_web::{web, HttpResponse};

pub async fn login_page() -> HttpResult {
    let mut context = tera::Context::new();
    context.insert("hide_top_btns", &false);
    render_template(&TEMPLATES, "login.html", &context)
}
#[derive(serde::Deserialize)]
pub(crate) struct LoginForm {
    #[serde(rename(deserialize = "password"))]
    pwd: String,
}

pub(crate) async fn login(
    id: Identity,
    form: web::Form<LoginForm>,
    data: web::Data<State>,
) -> HttpResult {
    let pwd = form.into_inner().pwd;
    if data.pwd == hash(pwd) {
        id.remember("auth".to_string());
        Ok(redirect("/"))
    } else {
        Err(HttpError::WrongPassword)
    }
}

pub async fn logout(id: Identity) -> HttpResponse {
    id.forget();
    redirect("/login")
}
