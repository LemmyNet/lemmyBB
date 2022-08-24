use crate::{
    api,
    api::{site::get_site, user::get_captcha},
    routes::{auth, site::rocket_uri_macro_index, ErrorPage},
};
use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    response::Redirect,
    Either,
};
use rocket_dyn_templates::{context, Template};

#[get("/login")]
pub async fn login(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = get_site(auth(cookies)).await?;
    Ok(Template::render("login", context!(site)))
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
pub async fn register() -> Result<Template, ErrorPage> {
    let site = get_site(None).await?;
    let captcha = get_captcha().await?;
    Ok(Template::render("register", context!(site, captcha)))
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

    let site = get_site(None).await?;
    let ctx = context!(site, message);
    Ok(Either::Left(Template::render("message", ctx)))
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    // simply delete the cookie
    cookies.remove(Cookie::named("jwt"));
    Ok(Redirect::to(uri!(index)))
}
