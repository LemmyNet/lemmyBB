use crate::{
    api::{
        comment::{create_comment, edit_comment, get_comment},
        post::get_post,
        site::get_site_data,
    },
    error::ErrorPage,
    routes::{auth, post::rocket_uri_macro_view_topic},
    utils::replace_smilies,
};
use rocket::{form::Form, http::CookieJar, response::Redirect, Either};
use rocket_dyn_templates::{context, Template};

#[get("/comment_editor?<t>&<edit>")]
pub async fn comment_editor(
    t: i32,
    edit: Option<i32>,
    cookies: &CookieJar<'_>,
) -> Result<Template, ErrorPage> {
    match edit {
        Some(e) => {
            let c = get_comment(e, auth(cookies)).await?;
            render_editor(t, Some(c.comment_view.comment.content), edit, cookies).await
        }
        None => render_editor(t, None, None, cookies).await,
    }
}

async fn render_editor(
    post_id: i32,
    message: Option<String>,
    edit_comment_id: Option<i32>,
    cookies: &CookieJar<'_>,
) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let post = get_post(post_id, auth(cookies)).await?;
    let mut editor_action = format!("/do_comment?t={}", post.post_view.post.id.0);
    if let Some(edit_comment_id) = edit_comment_id {
        editor_action = format!("{}&edit={}", editor_action, edit_comment_id);
    }
    let message = message.unwrap_or_default();
    Ok(Template::render(
        "comment_editor",
        context!(site_data, post, message, editor_action),
    ))
}

#[derive(FromForm)]
pub struct CommentForm {
    message: String,
    preview: Option<String>,
}

#[post("/do_comment?<t>&<edit>", data = "<form>")]
pub async fn do_comment(
    t: i32,
    edit: Option<i32>,
    form: Form<CommentForm>,
    cookies: &CookieJar<'_>,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let message = replace_smilies(&form.message, &site_data);
    if form.preview.is_some() {
        return Ok(Either::Left(
            render_editor(t, Some(message), edit, cookies).await?,
        ));
    }

    let auth = auth(cookies).unwrap();
    match edit {
        Some(e) => edit_comment(e, message, auth).await?,
        None => create_comment(t, message, auth).await?,
    };
    Ok(Either::Right(Redirect::to(uri!(view_topic(t, Some(1))))))
}
