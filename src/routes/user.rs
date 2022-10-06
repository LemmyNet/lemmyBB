use crate::{
    api,
    api::{
        private_message::{list_private_messages, mark_private_message_read},
        site::get_site_data,
        user::{get_captcha, get_person, mark_all_as_read},
    },
    routes::{auth, site::rocket_uri_macro_index, ErrorPage},
};
use chrono::NaiveDateTime;
use futures::future::join_all;
use itertools::Itertools;
use lemmy_api_common::person::PrivateMessageResponse;
use lemmy_db_schema::{newtypes::PersonId, source::person::PersonSafe};
use lemmy_db_views::structs::PrivateMessageView;
use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    response::Redirect,
    Either,
};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use std::hash::{Hash, Hasher};

#[get("/login")]
pub async fn login(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    Ok(Template::render("login", context!(site_data)))
}

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[post("/do_login", data = "<form>")]
pub async fn do_login(
    form: Form<LoginForm>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    let jwt = api::user::login(&form.username, &form.password)
        .await?
        .jwt
        .unwrap()
        .into_inner();
    cookies.add(Cookie::new("jwt", jwt));
    Ok(Redirect::to(uri!(index)))
}

#[get("/register")]
pub async fn register(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let captcha = get_captcha().await?;
    Ok(Template::render("register", context!(site_data, captcha)))
}

#[derive(FromForm, Default)]
pub struct RegisterForm {
    pub username: String,
    pub password: String,
    pub password_verify: String,
    pub show_nsfw: bool,
    pub email: Option<String>,
    pub captcha_uuid: Option<String>,
    pub captcha_answer: Option<String>,
    pub honeypot: Option<String>,
    pub application_answer: Option<String>,
    pub refresh_captcha: Option<String>,
}

#[post("/do_register", data = "<form>")]
pub async fn do_register(
    mut form: Form<RegisterForm>,
    cookies: &CookieJar<'_>,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    if form.refresh_captcha.is_some() {
        // user requested new captcha, so reload page
        return Ok(Either::Right(Redirect::to(uri!(register))));
    }

    // empty fields gets parsed into Some(""), convert that to None
    form.captcha_answer = form.captcha_answer.clone().filter(|h| !h.is_empty());
    form.honeypot = form.honeypot.clone().filter(|h| !h.is_empty());
    form.email = form.email.clone().filter(|h| !h.is_empty());
    form.application_answer = form.application_answer.clone().filter(|h| !h.is_empty());

    let res = api::user::register(form.into_inner()).await?;
    let message = if let Some(jwt) = res.jwt {
        cookies.add(Cookie::new("jwt", jwt.into_inner()));
        "Registration successful"
    } else if res.verify_email_sent {
        "Registration successful, confirm your email address"
    } else {
        "Registration successful, wait for admin approval"
    };

    let site = get_site_data(cookies).await?;
    let ctx = context!(site, message);
    Ok(Either::Left(Template::render("message", ctx)))
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    // simply delete the cookie
    cookies.remove(Cookie::named("jwt"));
    Ok(Redirect::to(uri!(index)))
}

#[post("/mark_all_notifications_read")]
pub async fn mark_all_notifications_read(cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    mark_all_as_read(auth(cookies).unwrap()).await?;
    Ok(Redirect::to(uri!(index)))
}

#[get("/view_profile?<u>")]
pub async fn view_profile(u: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let person = get_person(PersonId(u), auth(cookies)).await?;
    let ctx = context!(site_data, person);
    Ok(Template::render("view_profile", ctx))
}

#[derive(Serialize)]
struct PrivateMessageThread {
    pub other_participant: PersonSafe,
    pub read: bool,
    pub last_message: NaiveDateTime,
}

// TODO: add these derives directly in lemmy and remove wrapper
#[derive(PartialEq, Debug)]
struct PersonSafeWrapper(PersonSafe);

impl Eq for PersonSafeWrapper {}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for PersonSafeWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.actor_id.hash(state)
    }
}

#[get("/private_messages_list")]
pub async fn private_messages_list(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let my_user_id = site_data
        .site
        .my_user
        .as_ref()
        .unwrap()
        .local_user_view
        .person
        .id;
    let private_message_threads: Vec<_> = list_private_messages(false, auth(cookies).unwrap())
        .await?
        .private_messages
        .into_iter()
        // group by the other user involved in this pm
        .into_group_map_by(|pm| {
            if my_user_id != pm.creator.id {
                PersonSafeWrapper(pm.creator.clone())
            } else {
                // HACK: convert from PersonSafeAlias1 to PersonSafe
                PersonSafeWrapper(
                    serde_json::from_str(&serde_json::to_string(&pm.recipient.clone()).unwrap())
                        .unwrap(),
                )
            }
        })
        .into_iter()
        .map(|(user, pms)| {
            PrivateMessageThread {
                other_participant: user.0,
                // messages sent by own user are always unread
                // https://github.com/LemmyNet/lemmy/issues/2484
                read: pms
                    .iter()
                    .filter(|pm| pm.creator.id != my_user_id)
                    .all(|pm| pm.private_message.read),
                last_message: pms
                    .iter()
                    .map(|pm| pm.private_message.published)
                    .max()
                    .unwrap(),
            }
        })
        // newest messages first
        .sorted_by_key(|pmt| -pmt.last_message.timestamp())
        .collect();
    let ctx = context!(site_data, private_message_threads);
    Ok(Template::render("private_messages_list", ctx))
}

#[get("/private_messages_thread?<u>")]
pub async fn private_messages_thread(
    u: i32,
    cookies: &CookieJar<'_>,
) -> Result<Template, ErrorPage> {
    let u = PersonId(u);
    let site_data = get_site_data(cookies).await?;
    let auth = auth(cookies).unwrap();
    // TODO: would be nice if lemmy api could query PMs involving given user
    let private_messages: Vec<PrivateMessageView> = list_private_messages(false, auth.clone())
        .await?
        .private_messages
        .into_iter()
        .filter(|pm| pm.creator.id == u || pm.recipient.id == u)
        .collect();

    // mark as read
    let my_user_id = site_data
        .site
        .my_user
        .as_ref()
        .unwrap()
        .local_user_view
        .person
        .id;
    join_all(
        private_messages
            .iter()
            .filter(|pm| !pm.private_message.read)
            // messages sent by own user are also errouneously marked as unread
            // https://github.com/LemmyNet/lemmy/issues/2484
            .filter(|pm| pm.creator.id != my_user_id)
            .map(|pm| mark_private_message_read(pm.private_message.id, auth.clone())),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<PrivateMessageResponse>, anyhow::Error>>()?;

    let ctx = context!(site_data, private_messages);
    Ok(Template::render("private_messages_thread", ctx))
}
