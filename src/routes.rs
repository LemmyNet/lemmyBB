use crate::{
    api::{get_post, get_site, list_posts, login, CLIENT},
    error::ErrorPage,
};
use reqwest::header::HeaderName;
use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    response::Redirect,
};
use rocket_dyn_templates::{context, Template};
use url::Url;

#[get("/")]
pub async fn view_forum() -> Result<Template, ErrorPage> {
    let site = get_site().await?.site_view.unwrap();
    let posts = list_posts().await?.posts;
    let ctx = context! { site, posts };
    Ok(Template::render("viewforum", ctx))
}

#[get("/viewtopic?<t>")]
pub async fn view_topic(t: i32) -> Result<Template, ErrorPage> {
    let site = get_site().await?.site_view.unwrap();
    let mut post = get_post(t).await?;
    post.comments
        .sort_by(|a, b| a.comment.published.cmp(&b.comment.published));
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
pub async fn login_page() -> Result<Template, ErrorPage> {
    let site = get_site().await?.site_view.unwrap();
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
    let jwt = login(&form.username, &form.password)
        .await?
        .jwt
        .unwrap()
        .into_inner();
    cookies.add(Cookie::new("jwt", jwt));
    Ok(Redirect::to(uri!(view_forum)))
}
