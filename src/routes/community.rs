use crate::{
    api::{community::get_community, post::list_posts, site::get_site_data},
    routes::{auth, ErrorPage},
};
use rocket::http::CookieJar;
use rocket_dyn_templates::{context, Template};

#[get("/viewforum?<f>")]
pub async fn view_forum(f: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let auth = auth(cookies);
    let posts = list_posts(f, 20, auth.clone()).await?.posts;
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
    let ctx = context! { site_data, community, posts };
    Ok(Template::render("viewforum", ctx))
}
