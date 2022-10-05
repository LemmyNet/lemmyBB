use crate::{
    api::{
        extra::{get_notifications, Notification},
        gen_request_url,
        get,
        handle_response,
        post,
        CLIENT,
    },
    routes::auth,
};
use anyhow::Error;
use chrono::Local;
use futures::future::join;
use lemmy_api_common::{
    sensitive::Sensitive,
    site::{
        CreateSite,
        GetSite,
        GetSiteResponse,
        ResolveObject,
        ResolveObjectResponse,
        Search,
        SearchResponse,
        SiteResponse,
    },
};
use rocket::http::{Cookie, CookieJar};

/// Don't use get() function here, so that we can directly inspect api response, and handle error
/// `not_logged_in`. This commonly happens during development when Lemmy database was wiped, but
/// cookie is still present in browser. In that case, delete jwt cookie.
pub async fn get_site(
    cookies: &CookieJar<'_>,
) -> Result<(GetSiteResponse, Vec<Notification>, String), Error> {
    let params = GetSite {
        auth: auth(cookies),
    };
    let mut res = CLIENT
        .get(&gen_request_url("/site"))
        .query(&params)
        .send()
        .await?;
    let mut status = res.status();
    let mut text = res.text().await?;
    // auth token is not valid, delete it
    if text == r#"{"error":"not_logged_in"}"# {
        cookies.remove(Cookie::named("jwt"));
        res = CLIENT.get(&gen_request_url("/site")).send().await?;
        status = res.status();
        text = res.text().await?;
    }
    let site = handle_response(text, status)?;

    let notifications = match auth(cookies) {
        Some(auth) => get_notifications(auth).await?,
        None => vec![],
    };

    let current_date_time = Local::now().naive_local().format("%c").to_string();
    Ok((site, notifications, current_date_time))
}

pub async fn create_site(
    name: String,
    description: Option<String>,
    auth: String,
) -> Result<SiteResponse, Error> {
    let params = CreateSite {
        name,
        description,
        auth: Sensitive::new(auth),
        ..Default::default()
    };
    post("/site", &params).await
}

pub async fn search(
    query: String,
    auth: Option<Sensitive<String>>,
) -> Result<SearchResponse, Error> {
    let search_params = Search {
        q: query.clone(),
        auth: auth.clone(),
        ..Default::default()
    };
    let resolve_params = ResolveObject { q: query, auth };
    let (search, resolve) = join(
        get("/search", &search_params),
        get("/resolve_object", &resolve_params),
    )
    .await;

    // ignore resolve errors, those will happen every time that query is not an apub id
    let (mut search, resolve): (SearchResponse, ResolveObjectResponse) =
        (search?, resolve.unwrap_or_default());
    // for simplicity, we just append result from resolve_object to search result
    if let Some(p) = resolve.post {
        search.posts.push(p)
    };
    if let Some(c) = resolve.comment {
        search.comments.push(c)
    };
    if let Some(c) = resolve.community {
        search.communities.push(c)
    };
    if let Some(p) = resolve.person {
        search.users.push(p)
    };
    Ok(search)
}
