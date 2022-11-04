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
use rocket::{
    http::CookieJar,
    request::{FromRequest, Outcome},
    response::Redirect,
    Either,
};
use std::{path::PathBuf, str::FromStr};

type ReturnType = Result<Either<Redirect, String>, ErrorPage>;

#[get("/c/<name>")]
pub async fn apub_community(
    name: String,
    accept: AcceptHeader,
    cookies: &CookieJar<'_>,
) -> ReturnType {
    if accept.0.starts_with("application/") {
        return foward_apub_fetch(format!("{}/c/{}", lemmy_backend(), name), accept).await;
    }
    let community = get_community(NameOrId::Name(name), auth(cookies)).await?;
    let f = community.community_view.community.id.0;
    Ok(Either::Left(Redirect::to(uri!(view_forum(
        f,
        Some(1),
        Option::<String>::None
    )))))
}

#[get("/u/<name>")]
pub async fn apub_user(name: String, accept: AcceptHeader, cookies: &CookieJar<'_>) -> ReturnType {
    if accept.0.starts_with("application/") {
        return foward_apub_fetch(format!("{}/u/{}", lemmy_backend(), name), accept).await;
    }
    let user = get_person(NameOrId::Name(name), auth(cookies)).await?;
    let u = user.person_view.person.id.0;
    Ok(Either::Left(Redirect::to(uri!(view_profile(u)))))
}

#[get("/post/<id>")]
pub async fn apub_post(id: i32, accept: AcceptHeader) -> ReturnType {
    if accept.0.starts_with("application/") {
        return foward_apub_fetch(format!("{}/post/{}", lemmy_backend(), id), accept).await;
    }
    Ok(Either::Left(Redirect::to(uri!(view_topic(id, Some(1))))))
}

#[get("/comment/<t>")]
pub async fn apub_comment(t: i32, accept: AcceptHeader, cookies: &CookieJar<'_>) -> ReturnType {
    if accept.0.starts_with("application/") {
        return foward_apub_fetch(format!("{}/comment/{}", lemmy_backend(), t), accept).await;
    }
    let comment = get_comment(t, auth(cookies)).await?;
    // TODO: figure out actual page
    Ok(Either::Left(Redirect::to(format!(
        "/view_topic?t={}&page=1#p{}",
        t, comment.comment_view.comment.id
    ))))
}

/// In case an activitypub object is being fetched, forward request to Lemmy backend
async fn foward_apub_fetch(url: String, accept: AcceptHeader) -> ReturnType {
    let res = CLIENT
        .get(url)
        .header("accept", accept.0)
        .send()
        .await?
        .text()
        .await?;
    Ok(Either::Right(res))
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
impl<'r> FromRequest<'r> for Headers<'r> {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(Headers(req.headers()))
    }
}

pub struct AcceptHeader(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AcceptHeader {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(AcceptHeader(
            req.headers()
                .get_one("accept")
                .unwrap_or_default()
                .to_string(),
        ))
    }
}
