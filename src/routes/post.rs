use crate::{
    api::{
        community::get_community,
        post::{create_post, get_post},
        site::get_site_data,
    },
    error::ErrorPage,
    routes::{auth, CLIENT},
    template_helpers::replace_smilies,
};
use reqwest::header::HeaderName;
use rocket::{form::Form, http::CookieJar, response::Redirect, Either};
use rocket_dyn_templates::{context, Template};
use url::Url;

#[get("/viewtopic?<t>")]
pub async fn view_topic(t: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let mut post = get_post(t, auth(cookies)).await?;

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

    let ctx = context! { site_data, post, is_image_url };
    Ok(Template::render("viewtopic", ctx))
}

#[get("/post?<f>")]
pub async fn post(f: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let community = get_community(f, auth(cookies)).await?;
    let ctx = context!(site_data, community);
    Ok(Template::render("thread_editor", ctx))
}

#[derive(FromForm)]
pub struct PostForm {
    subject: String,
    message: String,
    preview: Option<String>,
}

#[post("/do_post?<f>", data = "<form>")]
pub async fn do_post(
    f: i32,
    mut form: Form<PostForm>,
    cookies: &CookieJar<'_>,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    form.message = replace_smilies(&form.message);

    let community = get_community(f, auth(cookies)).await?;
    if form.preview.is_some() {
        let site_data = get_site_data(cookies).await?;
        let ctx = context!(site_data, community, subject: &form.subject, message: &form.message);
        return Ok(Either::Left(Template::render("thread_editor", ctx)));
    }

    let post = create_post(
        form.subject.clone(),
        form.message.clone(),
        community.community_view.community.id,
        auth(cookies).unwrap(),
    )
    .await?;
    Ok(Either::Right(Redirect::to(uri!(view_topic(
        post.post_view.post.id.0
    )))))
}
