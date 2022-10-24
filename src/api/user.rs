use crate::{
    api::{get, post, put, NameOrId},
    routes::user::RegisterForm,
};
use anyhow::Error;
use lemmy_api_common::{
    person::{
        ChangePassword,
        GetCaptchaResponse,
        GetPersonDetails,
        GetPersonDetailsResponse,
        GetPersonMentions,
        GetPersonMentionsResponse,
        GetReplies,
        GetRepliesResponse,
        Login,
        LoginResponse,
        MarkAllAsRead,
        Register,
        SaveUserSettings,
    },
    sensitive::Sensitive,
};
use lemmy_db_schema::newtypes::PersonId;

pub async fn get_person(
    name_or_id: NameOrId,
    auth: Option<Sensitive<String>>,
) -> Result<GetPersonDetailsResponse, Error> {
    let mut params = GetPersonDetails {
        auth,
        ..Default::default()
    };
    match name_or_id {
        NameOrId::Name(n) => params.username = Some(n),
        NameOrId::Id(c) => params.person_id = Some(PersonId(c)),
    }
    get("/user", &params).await
}

pub async fn login(username_or_email: &str, password: &str) -> Result<LoginResponse, Error> {
    let params = Login {
        username_or_email: Sensitive::new(username_or_email.to_string()),
        password: Sensitive::new(password.to_string()),
    };
    post("/user/login", &params).await
}

pub async fn get_captcha() -> Result<GetCaptchaResponse, Error> {
    get("/user/get_captcha", &()).await
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

pub(crate) async fn list_mentions(
    auth: Sensitive<String>,
) -> Result<GetPersonMentionsResponse, Error> {
    let params = GetPersonMentions {
        auth,
        unread_only: Some(true),
        ..Default::default()
    };
    get("/user/mention", &params).await
}

pub(crate) async fn list_replies(auth: Sensitive<String>) -> Result<GetRepliesResponse, Error> {
    let params = GetReplies {
        auth,
        unread_only: Some(true),
        ..Default::default()
    };
    get("/user/replies", &params).await
}

pub async fn mark_all_as_read(auth: Sensitive<String>) -> Result<GetRepliesResponse, Error> {
    let params = MarkAllAsRead { auth };
    post("/user/mark_all_as_read", &params).await
}

pub async fn change_password(params: ChangePassword) -> Result<LoginResponse, Error> {
    put("/user/change_password", &params).await
}
pub async fn save_settings(params: SaveUserSettings) -> Result<LoginResponse, Error> {
    put("/user/save_user_settings", &params).await
}
