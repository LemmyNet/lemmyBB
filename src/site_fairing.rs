use crate::{
    api::{
        extra::{get_notifications, Notification},
        gen_request_url,
        handle_response,
        private_message::list_private_messages,
        CLIENT,
    },
    routes::auth,
};
use anyhow::Error;
use chrono::Local;
use futures::future::join;
use itertools::Itertools;
use lemmy_api_common::{
    sensitive::Sensitive,
    site::{GetSite, GetSiteResponse},
};
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Cookie,
    request,
    request::FromRequest,
    Data,
    Request,
};
use serde::{Deserialize, Serialize};

pub struct SiteFairing {}

#[rocket::async_trait]
impl Fairing for SiteFairing {
    fn info(&self) -> Info {
        Info {
            name: "Site data fetcher",
            kind: Kind::Request,
        }
    }

    /// Load site data for everything except /assets paths
    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        // TODO: might not need this anymore as we have caching
        if !req.uri().path().starts_with("/assets") {
            let _: &Option<SiteData> = req
                .local_cache_async(async {
                    let site_data = get_site_data(req).await;
                    if let Err(e) = &site_data {
                        warn!("{}", e);
                    }
                    site_data.ok()
                })
                .await;
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SiteData {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, ()> {
        let site_data: &Option<SiteData> = request.local_cache(|| None::<SiteData>);
        request::Outcome::Success(site_data.clone().unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SiteData {
    pub site: GetSiteResponse,
    pub notifications: Vec<Notification>,
    pub unread_pm_count: usize,
    pub current_date_time: String,
    pub auth: Option<Sensitive<String>>,
    pub lang: String,
    pub lemmybb_version: String,
}

async fn get_site_data(request: &Request<'_>) -> Result<SiteData, Error> {
    let mut auth = auth(request.cookies());
    let params = GetSite { auth: auth.clone() };
    let res = CLIENT
        .get(&gen_request_url("/site"))
        .query(&params)
        .send()
        .await?;
    let site: GetSiteResponse = match handle_response(res, "/site").await {
        Ok(o) => o,
        Err(e) => {
            if e.to_string() == "not_logged_in" {
                // if auth cookie is invalid, remove it and retry
                request.cookies().remove(Cookie::named("jwt"));
                auth = None;
                let res = CLIENT
                    .get(&gen_request_url("/site"))
                    .query(&GetSite { auth: None })
                    .send()
                    .await?;
                handle_response(res, "/site").await?
            } else {
                Err(e)?
            }
        }
    };
    let browser_lang = request
        .headers()
        .get("accept-language")
        .collect_vec()
        .first()
        .unwrap_or(&"")
        .to_string();
    let lang = match &site.my_user {
        Some(u) => {
            let user_lang = u.local_user_view.local_user.interface_language.clone();
            match user_lang == "browser" {
                true => browser_lang,
                false => user_lang,
            }
        }
        None => browser_lang,
    };

    let mut site_data = SiteData {
        site,
        notifications: vec![],
        // TODO: why is this?
        unread_pm_count: 0,
        current_date_time: Local::now().naive_local().format("%a %v %R").to_string(),
        auth: auth.clone(),
        lang,
        lemmybb_version: option_env!("LEMMYBB_VERSION")
            .unwrap_or("unknown version")
            .to_string(),
    };
    if let Some(auth) = auth {
        let (notifications, private_messages) = join(
            get_notifications(auth.clone()),
            list_private_messages(true, auth.clone()),
        )
        .await;
        site_data.notifications = notifications?;
        site_data.unread_pm_count = private_messages?.private_messages.len();
    }
    Ok(site_data)
}

#[cfg(test)]
pub async fn test_site_data(auth: Option<Sensitive<String>>) -> SiteData {
    let params = GetSite { auth: auth.clone() };
    let res = CLIENT
        .get(&gen_request_url("/site"))
        .query(&params)
        .send()
        .await
        .unwrap();
    let site: GetSiteResponse = handle_response(res, "/site").await.unwrap();
    SiteData {
        site,
        notifications: vec![],
        unread_pm_count: 0,
        current_date_time: "".to_string(),
        auth,
        lang: "".to_string(),
        lemmybb_version: "".to_string(),
    }
}
