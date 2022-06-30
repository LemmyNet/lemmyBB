use anyhow::Error;
use lemmy_api_common::{
    lemmy_db_schema::{
        newtypes::{CommunityId, PostId},
        ListingType,
        SortType,
    },
    person::{Login, LoginResponse, Register},
    post::{CreatePost, GetPost, GetPostResponse, GetPosts, GetPostsResponse, PostResponse},
    sensitive::Sensitive,
    site::{CreateSite, GetSiteResponse, ResolveObject, ResolveObjectResponse, SiteResponse},
};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, time::Duration};

static LEMMY_BACKEND: &str = "http://localhost:8536";
static LEMMY_API_VERSION: &str = "/api/v3";

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .expect("build client")
});

fn gen_request_url(path: &str) -> String {
    format!("{}{}{}", LEMMY_BACKEND, LEMMY_API_VERSION, path)
}

pub async fn list_posts() -> Result<GetPostsResponse, Error> {
    let params = GetPosts {
        type_: Some(ListingType::All),
        sort: Some(SortType::New),
        ..Default::default()
    };
    get("/post/list", Some(params)).await
}

pub async fn get_post(id: i32) -> Result<GetPostResponse, Error> {
    let params = GetPost {
        id: PostId(id),
        auth: None,
    };
    get("/post", Some(params)).await
}

pub async fn create_post(title: &str, auth: Sensitive<String>) -> Result<PostResponse, Error> {
    let params = CreatePost {
        name: title.to_string(),
        community_id: CommunityId(2),
        auth,
        ..Default::default()
    };
    post("/post", &params).await
}

pub async fn get_site() -> Result<GetSiteResponse, Error> {
    get::<GetSiteResponse, ()>("/site", None).await
}

pub async fn create_site(auth: Sensitive<String>) -> Result<SiteResponse, Error> {
    let params = CreateSite {
        name: "lemmyBB".to_string(),
        description: Some("Welcome to lemmyBB, enjoy your stay!".to_string()),
        auth,
        ..Default::default()
    };
    post("/site", &params).await
}

pub async fn register() -> Result<LoginResponse, Error> {
    let pass = Sensitive::new("lemmylemmy".to_string());
    let params = Register {
        username: "lemmy".to_string(),
        password: pass.clone(),
        password_verify: pass,
        ..Default::default()
    };
    post("/user/register", &params).await
}

pub async fn login() -> Result<LoginResponse, Error> {
    let params = Login {
        username_or_email: Sensitive::new("lemmy".to_string()),
        password: Sensitive::new("lemmylemmy".to_string()),
    };
    post("/user/login", &params).await
}

pub async fn resolve_object(query: String) -> Result<ResolveObjectResponse, Error> {
    let params = ResolveObject {
        q: query,
        auth: None,
    };
    get("/resolve_object", Some(params)).await
}

async fn post<T, Params>(path: &str, params: Params) -> Result<T, Error>
where
    T: DeserializeOwned,
    Params: Serialize + Debug,
{
    info!("post {}, params {:?}", &path, &params);
    let res = CLIENT
        .post(&gen_request_url(path))
        .json(&params)
        .send()
        .await?
        .text()
        .await?;
    info!("post {} response: {}", &path, &res);
    Ok(serde_json::from_str(&res)?)
}

async fn get<T, Params>(path: &str, params: Option<Params>) -> Result<T, Error>
where
    T: DeserializeOwned,
    Params: Serialize + Debug,
{
    info!("get {}, params {:?}", &path, &params);
    let r = CLIENT.get(&gen_request_url(path));
    let r = match params {
        Some(p) => r.query(&p),
        None => r,
    };
    Ok(r.send().await?.json().await?)
}
