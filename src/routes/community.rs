use crate::{
    api::{post::list_posts, site::get_site_data},
    routes::{auth, ErrorPage},
};
use rocket::http::CookieJar;
use rocket_dyn_templates::{context, Template};

#[get("/viewforum?<f>")]
pub async fn view_forum(f: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let posts = list_posts(f, 20, auth(cookies)).await?.posts;
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
    let ctx = context! { site_data, f, posts };
    Ok(Template::render("viewforum", ctx))
}
