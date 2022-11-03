use crate::{
    api::{
        comment::report_comment,
        community::{follow_community, get_community},
        extra::{get_last_reply_in_thread, PostOrComment},
        post::{list_posts, report_post},
        NameOrId,
    },
    env::increased_rate_limit,
    pagination::{PageLimit, Pagination, PAGE_ITEMS},
    routes::ErrorPage,
    site_fairing::SiteData,
};
use anyhow::Error;
use futures::future::join_all;
use rocket::form::Form;
use rocket_dyn_templates::{context, Template};

#[get("/view_forum?<f>&<page>&<action>")]
pub async fn view_forum(
    f: i32,
    page: Option<i32>,
    action: Option<String>,
    site_data: SiteData,
) -> Result<Template, ErrorPage> {
    let auth = site_data.auth.clone();
    if let Some(action) = action {
        if action == "subscribe" {
            follow_community(f, true, auth.clone().unwrap()).await?;
        } else if action == "unsubscribe" {
            follow_community(f, false, auth.clone().unwrap()).await?;
        }
    }
    let page = page.unwrap_or(1);
    let posts = list_posts(f, PAGE_ITEMS, page, auth.clone()).await?.posts;
    let community = get_community(NameOrId::Id(f), auth.clone()).await?;
    let last_replies = if increased_rate_limit() {
        join_all(
            posts
                .iter()
                .map(|p| get_last_reply_in_thread(p, auth.clone())),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<PostOrComment>, Error>>()?
    } else {
        vec![]
    };

    let limit = PageLimit::Unknown(posts.len());
    let pagination = Pagination::new(page, limit, format!("/viewforum?f={}&", f));
    let ctx = context! { site_data, community, posts, last_replies, pagination };
    Ok(Template::render("view_forum", ctx))
}

#[get("/report?<thread>&<reply>")]
pub async fn report(
    thread: Option<i32>,
    reply: Option<i32>,
    site_data: SiteData,
) -> Result<Template, ErrorPage> {
    let action = if let Some(thread) = thread {
        format!("/do_report?thread={}", thread)
    } else if let Some(reply) = reply {
        format!("/do_report?reply={}", reply)
    } else {
        unreachable!()
    };
    let ctx = context! { site_data, action };
    Ok(Template::render("report", ctx))
}

#[derive(FromForm)]
pub struct ReportForm {
    report_text: String,
}

#[post("/do_report?<thread>&<reply>", data = "<form>")]
pub async fn do_report(
    thread: Option<i32>,
    reply: Option<i32>,
    form: Form<ReportForm>,
    site_data: SiteData,
) -> Result<Template, ErrorPage> {
    let auth = site_data.auth.clone().unwrap();
    if let Some(thread) = thread {
        report_post(thread, form.report_text.clone(), auth).await?;
    } else if let Some(reply) = reply {
        report_comment(reply, form.report_text.clone(), auth).await?;
    } else {
        unreachable!()
    };
    let message = "Report created";
    Ok(Template::render("message", context! { site_data, message }))
}
