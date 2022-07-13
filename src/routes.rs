use crate::{
    api::{
        create_comment,
        create_post,
        get_community,
        get_post,
        get_site,
        list_posts,
        login,
        CLIENT,
    },
    error::ErrorPage,
};
use lemmy_api_common::sensitive::Sensitive;
use reqwest::header::HeaderName;
use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    response::Redirect,
};
use rocket_dyn_templates::{context, Template};
use url::Url;

fn auth(cookies: &CookieJar<'_>) -> Option<Sensitive<String>> {
    cookies
        .get("jwt")
        .map(|c| Sensitive::new(c.value().to_string()))
}

#[get("/")]
pub async fn view_forum(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = get_site(auth(cookies)).await?;
    let posts = list_posts(auth(cookies)).await?.posts;
    posts.len();
    let ctx = context! { site, posts };
    Ok(Template::render("viewforum", ctx))
}

#[get("/viewtopic?<t>")]
pub async fn view_topic(t: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = get_site(auth(cookies)).await?;
    let mut post = get_post(t, auth(cookies)).await?;

    // show oldest comments first
    post.comments
        .sort_by(|a, b| a.comment.published.cmp(&b.comment.published));

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
pub async fn login_page(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
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
    let jwt = login(&form.username, &form.password)
        .await?
        .jwt
        .unwrap()
        .into_inner();
    cookies.add(Cookie::new("jwt", jwt));
    Ok(Redirect::to(uri!(view_forum)))
}

#[get("/post")]
pub async fn post(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = get_site(auth(cookies)).await?;
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
    let community = get_community(form.community_name.clone(), auth(cookies)).await?;
    let post = create_post(
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
    let site = get_site(auth(cookies)).await?;
    let post = get_post(t, auth(cookies)).await?;
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
    create_comment(t, form.message.clone(), auth(cookies).unwrap()).await?;
    Ok(Redirect::to(uri!(view_topic(t))))
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    // simply delete the cookie
    cookies.remove(Cookie::named("jwt"));
    Ok(Redirect::to(uri!(view_forum)))
}
