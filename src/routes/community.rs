use crate::{
    api::{
        extra::{get_last_reply_in_thread, PostOrComment},
        post::list_posts,
        site::get_site,
    },
    routes::{auth, ErrorPage},
};
use anyhow::Error;
use futures::future::join_all;
use rocket::http::CookieJar;
use rocket_dyn_templates::{context, Template};

#[get("/viewforum?<f>")]
pub async fn view_forum(f: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = get_site(cookies).await?;
    let posts = list_posts(f, 20, auth(cookies)).await?.posts;
    let last_replies = join_all(
        posts
            .iter()
            .map(|p| get_last_reply_in_thread(p, auth(cookies))),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<PostOrComment>, Error>>()?;
    let ctx = context! { site, f, posts, last_replies };
    Ok(Template::render("viewforum", ctx))
}
