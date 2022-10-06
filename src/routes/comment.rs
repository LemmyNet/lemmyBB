use crate::{
    api::{comment::create_comment, post::get_post, site::get_site_data},
    error::ErrorPage,
    routes::{auth, post::rocket_uri_macro_view_topic},
    template_helpers::replace_smilies,
};
use rocket::{form::Form, http::CookieJar, response::Redirect, Either};
use rocket_dyn_templates::{context, Template};

#[get("/comment?<t>")]
pub async fn comment(t: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    comment_with_preview(t, None, cookies).await
}

async fn comment_with_preview(
    post_id: i32,
    form: Option<CommentForm>,
    cookies: &CookieJar<'_>,
) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let post = get_post(post_id, auth(cookies)).await?;
    Ok(if let Some(form) = form {
        Template::render("editor", context!(site_data, post, message: form.message))
    } else {
        Template::render("editor", context!(site_data, post))
    })
}

#[derive(FromForm)]
pub struct CommentForm {
    message: String,
    preview: Option<String>,
}

#[post("/do_comment?<t>", data = "<form>")]
pub async fn do_comment(
    t: i32,
    mut form: Form<CommentForm>,
    cookies: &CookieJar<'_>,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    form.message = replace_smilies(&form.message);
    if form.preview.is_some() {
        return Ok(Either::Left(
            comment_with_preview(t, Some(form.into_inner()), cookies).await?,
        ));
    }
    create_comment(t, form.message.clone(), auth(cookies).unwrap()).await?;
    Ok(Either::Right(Redirect::to(uri!(view_topic(t)))))
}
