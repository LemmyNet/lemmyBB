use crate::{
    api::{
        comment::{create_comment, edit_comment, get_comment},
        post::get_post,
    },
    error::ErrorPage,
    rocket_uri_macro_login,
    routes::post::rocket_uri_macro_view_topic,
    site_fairing::SiteData,
    utils::replace_smilies,
};
use rocket::{form::Form, response::Redirect, Either};
use rocket_dyn_templates::{context, Template};

#[get("/comment_editor?<t>&<edit>")]
pub async fn comment_editor(
    t: i32,
    edit: Option<i32>,
    site_data: SiteData,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    if site_data.auth.is_none() {
        return Ok(Either::Right(Redirect::to(uri!(login))));
    }
    match edit {
        Some(e) => {
            let c = get_comment(e, site_data.auth.clone()).await?;
            Ok(Either::Left(
                render_editor(t, Some(c.comment_view.comment.content), edit, site_data).await?,
            ))
        }
        None => Ok(Either::Left(render_editor(t, None, None, site_data).await?)),
    }
}

async fn render_editor(
    post_id: i32,
    message: Option<String>,
    edit_comment_id: Option<i32>,
    site_data: SiteData,
) -> Result<Template, ErrorPage> {
    let post = get_post(post_id, site_data.auth.clone()).await?;
    let mut editor_action = format!("/comment?t={}", post.post_view.post.id.0);
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

#[post("/comment?<t>&<edit>", data = "<form>")]
pub async fn do_comment(
    t: i32,
    edit: Option<i32>,
    form: Form<CommentForm>,
    site_data: SiteData,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    let message = replace_smilies(&form.message, &site_data);
    if form.preview.is_some() {
        return Ok(Either::Left(
            render_editor(t, Some(message), edit, site_data).await?,
        ));
    }

    let auth = site_data.auth.expect("user not logged in");
    match edit {
        Some(e) => edit_comment(e, message, auth).await?,
        None => create_comment(t, message, auth).await?,
    };
    Ok(Either::Right(Redirect::to(uri!(view_topic(t, Some(1))))))
}
