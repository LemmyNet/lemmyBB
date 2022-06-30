use crate::{
    api::{get_post, get_site, list_posts, login},
    error::ErrorPage,
};
use lemmy_api_common::post::GetPostResponse;
use lemmy_db_views::structs::{PostView, SiteView};
use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    response::Redirect,
};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

#[derive(Serialize)]
struct ViewForumTemplate {
    site: SiteView,
    posts: Vec<PostView>,
}

#[get("/")]
pub async fn view_forum() -> Result<Template, ErrorPage> {
    let site = get_site().await?.site_view.unwrap();
    let posts = list_posts().await?.posts;
    let ctx = ViewForumTemplate { site, posts };
    Ok(Template::render("viewforum", ctx))
}

#[derive(Serialize)]
struct ViewTopicTemplate {
    site: SiteView,
    post: GetPostResponse,
}

#[get("/viewtopic?<t>")]
pub async fn view_topic(t: i32) -> Result<Template, ErrorPage> {
    let site = get_site().await?.site_view.unwrap();
    let mut post = get_post(t).await?;
    post.comments
        .sort_by(|a, b| a.comment.published.cmp(&b.comment.published));
    let ctx = ViewTopicTemplate { site, post };
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
