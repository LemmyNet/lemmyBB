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
    let site_data = get_site_data(cookies).await?;
    let post = get_post(t, auth(cookies)).await?;
    let ctx = context!(site_data, post);
    Ok(Template::render("comment_editor", ctx))
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
        let site_data = get_site_data(cookies).await?;
        let post = get_post(t, auth(cookies)).await?;
        let ctx = context!(site_data, post, message: &form.message);
        return Ok(Either::Left(Template::render("comment_editor", ctx)));
    }

    create_comment(t, form.message.clone(), auth(cookies).unwrap()).await?;
    Ok(Either::Right(Redirect::to(uri!(view_topic(t)))))
}
