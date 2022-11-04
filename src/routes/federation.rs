use crate::{
    api::{comment::get_comment, community::get_community, user::get_person, NameOrId, CLIENT},
    env::lemmy_backend,
    error::ErrorPage,
    rocket_uri_macro_view_forum,
    rocket_uri_macro_view_profile,
    rocket_uri_macro_view_topic,
    routes::auth,
};
use anyhow::Error;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rocket::{http::CookieJar, response::Redirect};
use std::{path::PathBuf, str::FromStr};

#[get("/c/<name>")]
pub async fn redirect_apub_community(
    name: String,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    let community = get_community(NameOrId::Name(name), auth(cookies)).await?;
    let f = community.community_view.community.id.0;
    Ok(Redirect::to(uri!(view_forum(
        f,
        Some(1),
        Option::<String>::None
    ))))
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

#[post("/<path..>", data = "<body>")]
pub async fn inboxes(
    path: PathBuf,
    body: String,
    headers: Headers<'_>,
) -> Result<String, ErrorPage> {
    let url = format!("{}/{}", lemmy_backend(), path.to_str().unwrap());
    let headers = headers
        .0
        .iter()
        .map(|h| {
            Ok((
                HeaderName::from_str(h.name.as_str())?,
                HeaderValue::from_str(&h.value)?,
            ))
        })
        .collect::<Result<HeaderMap, Error>>()?;

    Ok(CLIENT
        .post(url)
        .headers(headers)
        .body(body)
        .send()
        .await?
        .text()
        .await?)
}

// Retrieve request headers
// https://github.com/SergioBenitez/Rocket/issues/178#issuecomment-953370904
pub struct Headers<'r>(&'r rocket::http::HeaderMap<'r>);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for Headers<'r> {
    type Error = std::convert::Infallible;

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        rocket::request::Outcome::Success(Headers(req.headers()))
    }
}
