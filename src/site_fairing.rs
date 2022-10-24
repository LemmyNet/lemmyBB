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
    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        let site_data = get_site_data(request).await.unwrap();
        request.local_cache(|| RequestSiteData(Some(site_data)));
    }
}

/// Value stored in request-local state.
#[derive(Clone)]
struct RequestSiteData(Option<SiteData>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SiteData {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, ()> {
        let cached = request.local_cache(|| RequestSiteData(None));
        let site_data = match cached {
            RequestSiteData(Some(s)) => s.clone(),
            RequestSiteData(None) => get_site_data(request).await.unwrap(),
        };
        request::Outcome::Success(site_data)
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
}

/// Don't use get() function here, so that we can directly inspect api response, and handle error
/// `not_logged_in`. This commonly happens during development when Lemmy database was wiped, but
/// cookie is still present in browser. In that case, delete jwt cookie.
async fn get_site_data(request: &Request<'_>) -> Result<SiteData, Error> {
    let auth = auth(request.cookies());
    let params = GetSite { auth: auth.clone() };
    let res = CLIENT
        .get(&gen_request_url("/site"))
        .query(&params)
        .send()
        .await?;
    let site: GetSiteResponse = handle_response(res, "/site").await?;
    let browser_lang = request
        .headers()
        .get("accept-language")
        .collect_vec()
        .first()
        .unwrap_or(&"")
        .to_string();
    let lang = match &site.my_user {
        Some(u) => {
            let user_lang = u.local_user_view.local_user.lang.clone();
            match user_lang == "browser" {
                true => browser_lang,
                false => user_lang,
            }
        }
        None => browser_lang,
    };

    let current_date_time = Local::now().naive_local().format("%a %v %R").to_string();
    Ok(if let Some(auth) = auth {
        let (notifications, private_messages) = join(
            get_notifications(auth.clone()),
            list_private_messages(true, auth.clone()),
        )
        .await;
        SiteData {
            site,
            notifications: notifications?,
            unread_pm_count: private_messages?.private_messages.len(),
            current_date_time,
            auth: Some(auth),
            lang,
        }
    } else {
        SiteData {
            site,
            notifications: vec![],
            unread_pm_count: 0,
            current_date_time,
            auth: None,
            lang,
        }
    })
}
