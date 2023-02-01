use crate::{
    api::{
        comment::{create_comment, edit_comment, get_comment, list_comments},
        post::get_post,
    },
    error::ErrorPage,
    pagination::PAGE_ITEMS,
    rocket_uri_macro_login,
    routes::post::rocket_uri_macro_view_topic,
    site_fairing::SiteData,
    utils::{replace_smilies, Context},
};
use lemmy_api_common::lemmy_db_views::structs::CommentView;
use rocket::{form::Form, response::Redirect, Either};
use rocket_dyn_templates::{context, Template};

#[get("/comment_editor?<t>&<edit>&<reply>")]
pub async fn comment_editor(
    t: i32,
    edit: Option<i32>,
    reply: Option<i32>,
    site_data: SiteData,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    if site_data.auth.is_none() {
        return Ok(Either::Right(Redirect::to(uri!(login))));
    }
    match edit {
        Some(e) => {
            let c = get_comment(e, site_data.auth.clone()).await?;
            Ok(Either::Left(
                render_editor(
                    t,
                    Some(c.comment_view.comment.content),
                    edit,
                    reply,
                    site_data,
                )
                .await?,
            ))
        }
        None => Ok(Either::Left(
            render_editor(t, None, None, reply, site_data).await?,
        )),
    }
}

async fn render_editor(
    post_id: i32,
    message: Option<String>,
    edit_comment_id: Option<i32>,
    reply: Option<i32>,
    site_data: SiteData,
) -> Result<Template, ErrorPage> {
    let post = get_post(post_id, site_data.auth.clone()).await?;
    let mut editor_action = format!("/comment?t={}", post.post_view.post.id.0);
    if let Some(edit_comment_id) = edit_comment_id {
        editor_action = format!("{editor_action}&edit={edit_comment_id}");
    }
    if let Some(reply) = reply {
        editor_action = format!("{editor_action}&reply={reply}");
    }
    let message = message.unwrap_or_default();

    // for topic review
    let all_comments = list_comments(post.post_view.post.id, site_data.auth.clone()).await?;
    let page_comments: Vec<CommentView> = all_comments
        .iter()
        .rev()
        .take(PAGE_ITEMS as usize)
        .cloned()
        .collect();

    let ctx = Context::builder()
        .title(format!(
            "Post a reply - {}",
            site_data.site.site_view.site.name
        ))
        .site_data(site_data)
        .other(context! { post, page_comments, message, editor_action, all_comments })
        .build();
    Ok(Template::render("comment_editor", ctx))
}

#[derive(FromForm)]
pub struct CommentForm {
    message: String,
    preview: Option<String>,
}

/// t: post id where the comment is made
/// edit: if this is set, edit the comment with this id
/// reply: new comment is a reply to comment with this id
/// form: actual comment content
#[post("/comment?<t>&<edit>&<reply>", data = "<form>")]
pub async fn do_comment(
    t: i32,
    edit: Option<i32>,
    reply: Option<i32>,
    form: Form<CommentForm>,
    site_data: SiteData,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    let message = replace_smilies(&form.message, &site_data);
    if form.preview.is_some() {
        return Ok(Either::Left(
            render_editor(t, Some(message), edit, reply, site_data).await?,
        ));
    }

    let auth = site_data.auth.expect("user not logged in");
    match edit {
        Some(e) => edit_comment(e, message, auth).await?,
        None => create_comment(t, message, reply, auth).await?,
    };
    Ok(Either::Right(Redirect::to(uri!(view_topic(t, Some(1))))))
}
