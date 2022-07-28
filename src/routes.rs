use crate::{
    api,
    api::{PostOrComment, CLIENT},
    error::ErrorPage,
    Error,
};
use futures::future::join_all;
use lemmy_api_common::sensitive::Sensitive;
use reqwest::header::HeaderName;
use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    response::Redirect,
    Either,
};
use rocket_dyn_templates::{context, Template};
use url::Url;

fn auth(cookies: &CookieJar<'_>) -> Option<Sensitive<String>> {
    cookies
        .get("jwt")
        .map(|c| Sensitive::new(c.value().to_string()))
}

#[get("/")]
pub async fn index(cookies: &CookieJar<'_>) -> Result<Either<Redirect, Template>, ErrorPage> {
    let site = api::get_site(auth(cookies)).await?;
    if site.site_view.is_none() {
        // need to setup site
        return Ok(Either::Left(Redirect::to(uri!(setup))));
    }

    let mut communities = api::list_communities(auth(cookies)).await?;
    communities
        .communities
        .sort_unstable_by_key(|c| c.community.id.0);
    let last_replies = join_all(
        communities
            .communities
            .iter()
            .map(|c| api::get_last_reply_in_community(c.community.id, auth(cookies))),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<Option<PostOrComment>>, Error>>()?;
    let ctx = context! { site, communities, last_replies };
    Ok(Either::Right(Template::render("index", ctx)))
}

#[get("/setup")]
pub async fn setup(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = api::get_site(auth(cookies)).await?;
    let ctx = context! { site };
    Ok(Template::render("setup", ctx))
}

#[derive(FromForm)]
pub struct SetupForm {
    // TODO:
    // #[serde(flatten)
    // register_form: RegisterForm,
    pub username: String,
    pub password: String,
    pub password_verify: String,
    pub show_nsfw: bool,
    pub email: Option<String>,
    pub site_name: String,
    pub site_description: Option<String>,
}

#[post("/do_setup", data = "<form>")]
pub async fn do_setup(
    form: Form<SetupForm>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    let register_form = RegisterForm {
        username: form.username.clone(),
        password: form.password.clone(),
        password_verify: form.password_verify.clone(),
        show_nsfw: form.show_nsfw,
        ..Default::default()
    };
    let jwt = api::register(register_form)
        .await?
        .jwt
        .unwrap()
        .into_inner();
    cookies.add(Cookie::new("jwt", jwt.clone()));

    api::create_site(form.site_name.clone(), form.site_description.clone(), jwt).await?;

    return Ok(Redirect::to(uri!(index)));
}

#[get("/viewforum?<f>")]
pub async fn view_forum(f: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = api::get_site(auth(cookies)).await?;
    let posts = api::list_posts(f, 20, auth(cookies)).await?.posts;
    let last_replies = join_all(
        posts
            .iter()
            .map(|p| api::get_last_reply_in_thread(p, auth(cookies))),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<PostOrComment>, Error>>()?;
    let ctx = context! { site, posts, last_replies };
    Ok(Template::render("viewforum", ctx))
}

#[get("/viewtopic?<t>")]
pub async fn view_topic(t: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = api::get_site(auth(cookies)).await?;
    let mut post = api::get_post(t, auth(cookies)).await?;

    // simply ignore deleted/removed comments
    post.comments = post
        .comments
        .into_iter()
        .filter(|c| !c.comment.deleted && !c.comment.removed)
        .collect();

    // determine if post.url should be rendered as <img> or <a href>
    let mut is_image_url = false;
    if let Some(ref url) = post.post_view.post.url {
        // TODO: use HEAD request once that is supported by pictrs/lemmy
        let image = CLIENT.get::<Url>(url.clone().into()).send().await?;
        let content_type = &image.headers()[HeaderName::from_static("content-type")];
        is_image_url = content_type.to_str()?.starts_with("image/");
    }

    let ctx = context! { site, post, is_image_url };
    Ok(Template::render("viewtopic", ctx))
}

#[get("/login")]
pub async fn login(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = api::get_site(auth(cookies)).await?;
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
    let jwt = api::login(&form.username, &form.password)
        .await?
        .jwt
        .unwrap()
        .into_inner();
    cookies.add(Cookie::new("jwt", jwt));
    Ok(Redirect::to(uri!(index)))
}

#[get("/register")]
pub async fn register() -> Result<Template, ErrorPage> {
    let site = api::get_site(None).await?;
    let captcha = api::get_captcha().await?;
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

    let res = api::register(form.into_inner()).await?;
    let message = if let Some(jwt) = res.jwt {
        cookies.add(Cookie::new("jwt", jwt.into_inner()));
        "Registration successful"
    } else if res.verify_email_sent {
        "Registration successful, confirm your email address"
    } else {
        "Registration successful, wait for admin approval"
    };

    let site = api::get_site(None).await?;
    let ctx = context!(site, message);
    Ok(Either::Left(Template::render("message", ctx)))
}

#[get("/post")]
pub async fn post(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = api::get_site(auth(cookies)).await?;
    Ok(Template::render("editor", context!(site)))
}

#[derive(FromForm)]
pub struct PostForm {
    subject: String,
    message: String,
    community_name: String,
}

#[post("/do_post", data = "<form>")]
pub async fn do_post(form: Form<PostForm>, cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    let community = api::get_community(form.community_name.clone(), auth(cookies)).await?;
    let post = api::create_post(
        form.subject.clone(),
        form.message.clone(),
        community.community_view.community.id,
        auth(cookies).unwrap(),
    )
    .await?;
    Ok(Redirect::to(uri!(view_topic(post.post_view.post.id.0))))
}

#[get("/comment?<t>")]
pub async fn comment(t: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = api::get_site(auth(cookies)).await?;
    let post = api::get_post(t, auth(cookies)).await?;
    Ok(Template::render("editor", context!(site, post)))
}

#[derive(FromForm)]
pub struct CommentForm {
    message: String,
}

#[post("/do_comment?<t>", data = "<form>")]
pub async fn do_comment(
    t: i32,
    form: Form<CommentForm>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    api::create_comment(t, form.message.clone(), auth(cookies).unwrap()).await?;
    Ok(Redirect::to(uri!(view_topic(t))))
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    // simply delete the cookie
    cookies.remove(Cookie::named("jwt"));
    Ok(Redirect::to(uri!(index)))
}
