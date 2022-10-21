use crate::{
    api::{
        extra::{get_notifications, Notification},
        gen_request_url,
        get,
        handle_response,
        post,
        private_message::list_private_messages,
        CLIENT,
    },
    env::lemmy_backend,
    error::ErrorPage,
    routes::auth,
    utils::base_url,
};
use anyhow::{anyhow, Error};
use chrono::Local;
use futures::future::join;
use image::{
    imageops::{resize, FilterType},
    io::Reader,
    ImageOutputFormat,
};
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
use lemmy_db_schema::newtypes::DbUrl;
use lemmy_db_views::structs::SiteView;
use once_cell::sync::OnceCell;
use rand::distributions::{Alphanumeric, DistString};
use reqwest::multipart::{Form, Part};
use rocket::{
    fs::TempFile,
    http::{Cookie, CookieJar, Status},
    response::Responder,
    Request,
    Response,
};
use serde::{Deserialize, Serialize};
use std::{
    env::temp_dir,
    fs::File,
    io::{Cursor, Read},
};
use url::Url;

#[derive(Serialize)]
pub struct SiteData {
    pub site: GetSiteResponse,
    pub notifications: Vec<Notification>,
    pub unread_pm_count: usize,
    pub current_date_time: String,
}

/// Don't use get() function here, so that we can directly inspect api response, and handle error
/// `not_logged_in`. This commonly happens during development when Lemmy database was wiped, but
/// cookie is still present in browser. In that case, delete jwt cookie.
pub async fn get_site_data(cookies: &CookieJar<'_>) -> Result<SiteData, Error> {
    let auth = auth(cookies);
    let params = GetSite { auth: auth.clone() };
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
    let site: GetSiteResponse = handle_response(text, status)?;

    let current_date_time = Local::now().naive_local().format("%a %v %R").to_string();
    Ok(if let Some(auth) = auth {
        let (notifications, private_messages) = join(
            get_notifications(auth.clone()),
            list_private_messages(true, auth),
        )
        .await;
        SiteData {
            site,
            notifications: notifications?,
            unread_pm_count: private_messages?.private_messages.len(),
            current_date_time,
        }
    } else {
        SiteData {
            site,
            notifications: vec![],
            unread_pm_count: 0,
            current_date_time,
        }
    })
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
    let (search, resolve) = join(get("/search", &search_params), resolve_object(query, auth)).await;

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

pub async fn resolve_object(
    query: String,
    auth: Option<Sensitive<String>>,
) -> Result<ResolveObjectResponse, Error> {
    let resolve_params = ResolveObject { q: query, auth };
    get("/resolve_object", &resolve_params).await
}

static FAVICON: OnceCell<Favicon> = OnceCell::new();

#[derive(Debug)]
pub struct Favicon {
    bytes: Vec<u8>,
    url: Option<DbUrl>,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for &'o Favicon {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'o> {
        let mut res = Response::build();
        if self.bytes.is_empty() {
            res.status(Status::NotFound);
        } else {
            res.sized_body(None, Cursor::new(&self.bytes));
        }
        Ok(res.finalize())
    }
}

#[get("/favicon.png")]
pub async fn favicon() -> Result<&'static Favicon, ErrorPage> {
    let site: GetSiteResponse = get("/site", GetSite::default()).await?;
    if let Some(f) = FAVICON.get() {
        // update favicon if url changed
        if let Some(site_view) = site.site_view {
            if site_view.site.icon != f.url {
                generate_favicon(Some(site_view)).await?;
            }
        }
        Ok(f)
    } else {
        generate_favicon(site.site_view).await?;
        Ok(FAVICON.get().unwrap())
    }
}

async fn generate_favicon(site_view: Option<SiteView>) -> Result<(), Error> {
    let f = if let Some(url) = site_view.and_then(|s| s.site.icon) {
        let url2: Url = url.clone().into();
        let icon_bytes = CLIENT.get(url2).send().await?.bytes().await?;
        let image = Reader::new(Cursor::new(icon_bytes))
            .with_guessed_format()?
            .decode()?;
        let resized = resize(&image, 64, 64, FilterType::Gaussian);

        let mut bytes: Vec<u8> = Vec::new();
        resized.write_to(&mut Cursor::new(&mut bytes), ImageOutputFormat::Png)?;
        Favicon {
            bytes,
            url: Some(url),
        }
    } else {
        Favicon {
            bytes: vec![],
            url: None,
        }
    };

    FAVICON
        .set(f)
        .map_err(|_| anyhow!("failed to init favicon"))?;
    Ok(())
}

#[derive(Deserialize)]
pub struct UploadImageResponse {
    msg: String,
    files: Vec<UploadImageFile>,
}

#[derive(Deserialize)]
struct UploadImageFile {
    pub file: String,
    #[allow(dead_code)]
    pub delete_token: String,
}

pub async fn upload_image(
    image: &mut TempFile<'_>,
    auth: Sensitive<String>,
    site_data: &SiteData,
) -> Result<Url, Error> {
    // TODO: currently need to persist tempfile to be able to read bytes
    // https://github.com/SergioBenitez/Rocket/issues/2148
    let filename = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    let tempfile = format!("{}/{}", temp_dir().as_path().display(), filename);
    let file_name = image.raw_name().unwrap().as_str().unwrap().to_string();
    let mime_str = image.content_type().unwrap().to_string();
    image.persist_to(&tempfile).await?;
    let mut file = File::open(&tempfile)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let part = Part::bytes(data).file_name(file_name).mime_str(&mime_str)?;
    let form = Form::new().part("images[]", part);
    let path = format!("{}/pictrs/image", lemmy_backend());
    let res = CLIENT
        .post(&path)
        .header("cookie", format!("jwt={}", auth.into_inner()))
        .multipart(form)
        .send()
        .await?;
    let status = res.status();
    let text = res.text().await?;
    info!("post /pictrs/image status: {}, response: {}", status, &text);
    let res: UploadImageResponse = handle_response(text, status)?;
    if res.msg != "ok" {
        return Err(anyhow!(res.msg));
    }
    let filename = &res.files[0].file;
    Ok(Url::parse(&format!(
        "{}/pictrs/image/{}?thumbnail=120",
        base_url(site_data),
        filename
    ))?)
}
