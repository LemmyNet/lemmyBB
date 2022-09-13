use crate::{
    api::{
        extra::{get_notifications, Notification},
        gen_request_url,
        handle_response,
        post,
        CLIENT,
    },
    routes::auth,
};
use anyhow::Error;
use lemmy_api_common::{
    sensitive::Sensitive,
    site::{CreateSite, GetSite, GetSiteResponse, SiteResponse},
};
use rocket::http::{Cookie, CookieJar};

/// Don't use get() function here, so that we can directly inspect api response, and handle error
/// `not_logged_in`. This commonly happens during development when Lemmy database was wiped, but
/// cookie is still present in browser. In that case, delete jwt cookie.
pub async fn get_site(
    cookies: &CookieJar<'_>,
) -> Result<(GetSiteResponse, Vec<Notification>), Error> {
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
    Ok((site, notifications))
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
