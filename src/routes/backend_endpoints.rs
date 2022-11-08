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
    http::{CookieJar, Header},
    request::{FromRequest, Outcome},
    response::Redirect,
    Either,
};
use std::{collections::HashMap, path::PathBuf, str::FromStr};

type ReturnType = Result<Either<Redirect, BackendResponse>, ErrorPage>;

/// Fetch apub community
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

/// Fetch apub user
#[get("/u/<name>")]
pub async fn apub_user(name: String, accept: AcceptHeader, cookies: &CookieJar<'_>) -> ReturnType {
    if accept.0.starts_with("application/") {
        return foward_apub_fetch(format!("{}/u/{}", lemmy_backend(), name), accept).await;
    }
    let user = get_person(NameOrId::Name(name), auth(cookies)).await?;
    let u = user.person_view.person.id.0;
    Ok(Either::Left(Redirect::to(uri!(view_profile(u)))))
}

/// Fetch apub post
#[get("/post/<id>")]
pub async fn apub_post(id: i32, accept: AcceptHeader) -> ReturnType {
    if accept.0.starts_with("application/") {
        return foward_apub_fetch(format!("{}/post/{}", lemmy_backend(), id), accept).await;
    }
    Ok(Either::Left(Redirect::to(uri!(view_topic(id, Some(1))))))
}

/// Fetch apub comment
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

async fn foward_apub_fetch(url: String, accept: AcceptHeader) -> ReturnType {
    Ok(Either::Right(
        forward_get_request(url, accept, HashMap::new()).await?,
    ))
}

#[derive(Responder)]
pub struct BackendResponse {
    text: String,
    header: Header<'static>,
}

/// In case an activitypub object is being fetched, forward request to Lemmy backend
pub async fn forward_get_request(
    url: String,
    accept: AcceptHeader,
    query: HashMap<String, String>,
) -> Result<BackendResponse, ErrorPage> {
    let res = CLIENT
        .get(url)
        .header("accept", accept.0)
        .query(&query)
        .send()
        .await?;
    let content_type = res
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()?
        .to_string();
    let text = res.text().await?;
    Ok(BackendResponse {
        text,
        header: Header::new("content-type", content_type),
    })
}

/// Incoming federation requests
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

pub struct AcceptHeader(pub String);

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

/// RSS feeds
#[get("/feeds/<path..>?<query..>")]
pub async fn feeds(
    path: PathBuf,
    query: HashMap<String, String>,
    accept: AcceptHeader,
) -> Result<BackendResponse, ErrorPage> {
    let url = format!("{}/feeds/{}", lemmy_backend(), path.to_str().unwrap());
    forward_get_request(url, accept, query).await
}

/// well-known endpoints, used for webfinger and resolving nodeinfo endpoint.
#[get("/.well-known/<path..>?<query..>")]
pub async fn well_known(
    path: PathBuf,
    query: HashMap<String, String>,
    accept: AcceptHeader,
) -> Result<BackendResponse, ErrorPage> {
    let url = format!("{}/.well-known/{}", lemmy_backend(), path.to_str().unwrap());
    forward_get_request(url, accept, query).await
}

/// Federated node metadata, necessary for statistics crawlers.
#[get("/nodeinfo/<path..>")]
pub async fn node_info(path: PathBuf, accept: AcceptHeader) -> Result<BackendResponse, ErrorPage> {
    let url = format!("{}/nodeinfo/{}", lemmy_backend(), path.to_str().unwrap());
    forward_get_request(url, accept, HashMap::new()).await
}

/// Site metadata, necessary for lemmy crawler. Note that jwt cookie is not passed through.
#[get("/api/v3/site")]
pub async fn api_site(accept: AcceptHeader) -> Result<BackendResponse, ErrorPage> {
    let url = format!("{}/api/v3/site", lemmy_backend());
    forward_get_request(url, accept, HashMap::new()).await
}
