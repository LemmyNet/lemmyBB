use crate::{
    api::{
        community::get_community,
        post::{create_post, edit_post, get_post},
        NameOrId,
    },
    error::ErrorPage,
    pagination::{PageLimit, Pagination, PAGE_ITEMS},
    routes::CLIENT,
    site_fairing::SiteData,
    utils::replace_smilies,
};
use reqwest::header::HeaderName;
use rocket::{form::Form, response::Redirect, Either};
use rocket_dyn_templates::{context, Template};
use url::Url;

#[get("/view_topic?<t>&<page>")]
pub async fn view_topic(
    t: i32,
    page: Option<i32>,
    site_data: SiteData,
) -> Result<Template, ErrorPage> {
    let mut post = get_post(t, site_data.auth.clone()).await?;

    // simply ignore deleted/removed comments
    post.comments = post
        .comments
        .into_iter()
        .filter(|c| !c.comment.deleted && !c.comment.removed)
        .collect();
    post.comments.sort_by_key(|c| c.comment.published);
    let all_comments = post.comments.clone();
    post.comments = post
        .comments
        .into_iter()
        // select items for current page
        .skip(((page.unwrap_or(1) - 1) * PAGE_ITEMS) as usize)
        .take(PAGE_ITEMS as usize)
        .collect();

    // determine if post.url should be rendered as <img> or <a href>
    let mut is_image_url = false;
    if let Some(ref url) = post.post_view.post.url {
        // TODO: use HEAD request once that is supported by pictrs/lemmy
        let image = CLIENT.get::<Url>(url.clone().into()).send().await?;
        let content_type = &image.headers()[HeaderName::from_static("content-type")];
        is_image_url = content_type.to_str()?.starts_with("image/");
    }
    let limit = PageLimit::Known((all_comments.len() as f32 / PAGE_ITEMS as f32).ceil() as i32);
    let pagination = Pagination::new(page.unwrap_or(1), limit, &format!("/viewtopic?t={}&", t));

    let ctx = context! { site_data, post, is_image_url, all_comments, pagination };
    Ok(Template::render("view_topic", ctx))
}

#[get("/post_editor?<f>&<edit>")]
pub async fn post_editor(
    f: i32,
    edit: Option<i32>,
    site_data: SiteData,
) -> Result<Template, ErrorPage> {
    match edit {
        Some(e) => {
            let p = get_post(e, site_data.auth.clone()).await?.post_view.post;
            render_editor(
                f,
                Some((p.name, p.body.unwrap_or_default())),
                edit,
                site_data,
            )
            .await
        }
        None => render_editor(f, None, None, site_data).await,
    }
}

async fn render_editor(
    community_id: i32,
    subject_and_message: Option<(String, String)>,
    edit_post_id: Option<i32>,
    site_data: SiteData,
) -> Result<Template, ErrorPage> {
    let community = get_community(NameOrId::Id(community_id), site_data.auth.clone()).await?;
    let mut editor_action = format!("/post?f={}", community.community_view.community.id.0);
    if let Some(edit_post_id) = edit_post_id {
        editor_action = format!("{}&edit={}", editor_action, edit_post_id);
    }
    let subject = subject_and_message
        .as_ref()
        .map(|s| s.0.clone())
        .unwrap_or_default();
    let message = subject_and_message
        .as_ref()
        .map(|s| s.1.clone())
        .unwrap_or_default();
    Ok(Template::render(
        "thread_editor",
        context!(site_data, community, editor_action, subject, message),
    ))
}

#[derive(FromForm)]
pub struct PostForm {
    subject: String,
    message: String,
    preview: Option<String>,
}

#[post("/post?<f>&<edit>", data = "<form>")]
pub async fn do_post(
    f: i32,
    edit: Option<i32>,
    form: Form<PostForm>,
    site_data: SiteData,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    let subject = form.subject.clone();
    let message = replace_smilies(&form.message, &site_data);

    if form.preview.is_some() {
        return Ok(Either::Left(
            render_editor(f, Some((subject, message)), edit, site_data).await?,
        ));
    }

    let auth = site_data.auth.unwrap();
    let post = match edit {
        None => create_post(subject, message, f, auth).await?,
        Some(e) => edit_post(subject, message, e, auth).await?,
    };
    Ok(Either::Right(Redirect::to(uri!(view_topic(
        post.post_view.post.id.0,
        Some(1)
    )))))
}
