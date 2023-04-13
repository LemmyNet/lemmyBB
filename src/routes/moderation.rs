use crate::{
    api::{
        comment::get_comment,
        moderation::{get_mod_log, remove_comment, remove_post},
        post::get_post,
    },
    error::ErrorPage,
    site_fairing::SiteData,
    template_helpers::i18n_,
    utils::Context,
};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use comrak::{markdown_to_html, ComrakOptions};
use itertools::Itertools;
use lemmy_api_common::lemmy_db_schema::source::community::CommunitySafe;
use rocket::{form::Form, response::Redirect, Either};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

#[get("/remove_item?<t>&<r>")]
pub async fn remove_item(
    t: Option<i32>,
    r: Option<i32>,
    site_data: SiteData,
) -> Result<Template, ErrorPage> {
    if t.is_some() == r.is_some() {
        return Err(anyhow!("One of params t and r needs to be set").into());
    }
    let ctx = Context::builder()
        .title("Delete item")
        .site_data(site_data)
        .other(context! { t, r })
        .build();
    Ok(Template::render("remove_item", ctx))
}

#[derive(FromForm)]
pub struct RemoveItemForm {
    t: Option<i32>,
    r: Option<i32>,
    delete_reason: String,
    cancel: Option<String>,
}

#[post("/do_remove_item", data = "<form>")]
pub async fn do_remove_item<'r>(
    form: Form<RemoveItemForm>,
    site_data: SiteData,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    let auth = site_data.auth.clone().unwrap();
    let link_url = match (form.t, form.r) {
        (Some(t), None) => {
            get_post(t, site_data.auth.clone())
                .await?
                .post_view
                .post
                .ap_id
        }
        (None, Some(r)) => {
            get_comment(r, site_data.auth.clone())
                .await?
                .comment_view
                .comment
                .ap_id
        }
        _ => return Err(anyhow!("One of params t and r needs to be set").into()),
    };
    if form.cancel.is_some() {
        // cancelled, redirect back
        return Ok(Either::Right(Redirect::to(link_url.to_string())));
    }
    match (form.t, form.r) {
        (Some(t), None) => {
            remove_post(t, form.delete_reason.clone(), auth).await?;
        }
        (None, Some(r)) => {
            remove_comment(r, form.delete_reason.clone(), auth).await?;
        }
        _ => return Err(anyhow!("Invalid parameters").into()),
    };
    let message = "Item deleted successfully";
    let link_text = "Click here to return";
    let ctx = Context::builder()
        .title(message)
        .site_data(site_data)
        .other(context! { message, link_text, link_url })
        .build();
    Ok(Either::Left(Template::render("message", ctx)))
}

#[get("/mod_log")]
pub async fn mod_log(site_data: SiteData) -> Result<Template, ErrorPage> {
    let mod_log = get_mod_log(site_data.auth.clone()).await?;
    // TODO: consider moving this upstream
    let entries: Vec<Vec<ModLogEntry>> = vec![
        mod_log
            .removed_posts
            .into_iter()
            .map(|m| {
                // TODO: why is removed an option??
                let action = if m.mod_remove_post.removed.unwrap() {
                    "Removed"
                } else {
                    "Restored"
                };
                let message = format!("{action} post [{}]({})", m.post.name, m.post.ap_id);
                ModLogEntry {
                    community: Some(m.community),
                    reason: m.mod_remove_post.reason,
                    when: m.mod_remove_post.when_,
                    message,
                }
            })
            .collect(),
        mod_log
            .removed_comments
            .into_iter()
            .map(|m| {
                let action = if m.mod_remove_comment.removed.unwrap() {
                    "Removed"
                } else {
                    "Restored"
                };
                let mut content = m.comment.content.replace('\n', " ");
                if content.chars().count() > 100 {
                    content = format!("{}...", content.chars().take(100).collect::<String>());
                }
                let message = format!("{action} comment [{}]({})", content, m.comment.ap_id);
                ModLogEntry {
                    community: Some(m.community),
                    reason: m.mod_remove_comment.reason,
                    when: m.mod_remove_comment.when_,
                    message,
                }
            })
            .collect(),
    ];
    let entries: Vec<_> = entries
        .into_iter()
        .flatten()
        .map(|mut e| {
            e.message = markdown_to_html(&e.message, &ComrakOptions::default());
            e
        })
        .sorted_by_key(|e| e.when)
        .rev()
        .collect();

    let ctx = Context::builder()
        .title(i18n_(&site_data, "mod_log_title"))
        .site_data(site_data)
        .other(context! { entries })
        .build();
    Ok(Template::render("site/mod_log", ctx))
}

#[derive(Debug, Serialize)]
pub struct ModLogEntry {
    community: Option<CommunitySafe>,
    reason: Option<String>,
    when: NaiveDateTime,
    message: String,
}
