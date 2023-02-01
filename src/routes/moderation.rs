use crate::{
    api::{
        comment::get_comment,
        moderation::{remove_comment, remove_post},
        post::get_post,
    },
    error::ErrorPage,
    site_fairing::SiteData,
    utils::Context,
};
use anyhow::anyhow;

use rocket::{form::Form, response::Redirect, Either};
use rocket_dyn_templates::{context, Template};

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
