use crate::{
    api::{get, post},
    routes::user::RegisterForm,
};
use anyhow::Error;
use lemmy_api_common::{
    person::{
        GetCaptchaResponse,
        GetPersonDetails,
        GetPersonDetailsResponse,
        Login,
        LoginResponse,
        Register,
    },
    sensitive::Sensitive,
};
use lemmy_db_schema::newtypes::PersonId;

pub async fn get_person(
    person_id: PersonId,
    auth: Option<Sensitive<String>>,
) -> Result<GetPersonDetailsResponse, Error> {
    let params = GetPersonDetails {
        person_id: Some(person_id),
        auth,
        ..Default::default()
    };
    get("/user", params).await
}

pub async fn login(username_or_email: &str, password: &str) -> Result<LoginResponse, Error> {
    let params = Login {
        username_or_email: Sensitive::new(username_or_email.to_string()),
        password: Sensitive::new(password.to_string()),
    };
    post("/user/login", &params).await
}

pub async fn get_captcha() -> Result<GetCaptchaResponse, Error> {
    get("/user/get_captcha", ()).await
}

pub async fn register(form: RegisterForm) -> Result<LoginResponse, Error> {
    let params = Register {
        username: form.username,
        password: Sensitive::new(form.password),
        password_verify: Sensitive::new(form.password_verify),
        show_nsfw: form.show_nsfw,
        email: form.email.map(Sensitive::new),
        captcha_uuid: form.captcha_uuid,
        captcha_answer: form.captcha_answer,
        honeypot: form.honeypot,
        answer: form.application_answer,
    };
    post("/user/register", &params).await
}
