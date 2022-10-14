use crate::{
    api::{community::get_community, post::list_posts, site::get_site_data},
    pagination::{PageLimit, Pagination, PAGE_ITEMS},
    routes::{auth, ErrorPage},
};
use rocket::http::CookieJar;
use rocket_dyn_templates::{context, Template};

#[get("/viewforum?<f>&<page>")]
pub async fn view_forum(
    f: i32,
    page: Option<i32>,
    cookies: &CookieJar<'_>,
) -> Result<Template, ErrorPage> {
    let page = page.unwrap_or(1);
    let site_data = get_site_data(cookies).await?;
    let auth = auth(cookies);
    let posts = list_posts(f, PAGE_ITEMS, page, auth.clone()).await?.posts;
    let community = get_community(f, auth).await?;
    /*
    let last_replies = join_all(
        posts
            .iter()
            .map(|p| get_last_reply_in_thread(p, auth(cookies))),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<PostOrComment>, Error>>()?;
    */
    let limit = PageLimit::Unknown(posts.len());
    let pagination = Pagination::new(page, limit, format!("/viewforum?f={}&", f));
    let ctx = context! { site_data, community, posts, pagination };
    Ok(Template::render("viewforum", ctx))
}
