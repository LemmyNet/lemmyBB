use crate::{
    api::{comment::get_comment, community::get_community, user::get_person, NameOrId},
    error::ErrorPage,
    rocket_uri_macro_view_forum,
    rocket_uri_macro_view_profile,
    rocket_uri_macro_view_topic,
    routes::auth,
};
use rocket::{http::CookieJar, response::Redirect};

#[get("/c/<name>")]
pub async fn redirect_apub_community(
    name: String,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    let community = get_community(NameOrId::Name(name), auth(cookies)).await?;
    let f = community.community_view.community.id.0;
    Ok(Redirect::to(uri!(view_forum(f, Some(1)))))
}

#[get("/u/<name>")]
pub async fn redirect_apub_user(
    name: String,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    let user = get_person(NameOrId::Name(name), auth(cookies)).await?;
    let u = user.person_view.person.id.0;
    Ok(Redirect::to(uri!(view_profile(u))))
}

#[get("/post/<id>")]
pub async fn redirect_apub_post(id: i32) -> Redirect {
    Redirect::to(uri!(view_topic(id, Some(1))))
}

#[get("/comment/<t>")]
pub async fn redirect_apub_comment(t: i32, cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    let comment = get_comment(t, auth(cookies)).await?;
    // TODO: figure out actual page
    Ok(Redirect::to(format!(
        "/viewtopic?t={}&page=1#p{}",
        t, comment.comment_view.comment.id
    )))
}
