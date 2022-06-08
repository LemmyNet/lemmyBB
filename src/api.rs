use anyhow::Error;
use lemmy_api_common::{
    lemmy_db_schema::{newtypes::CommunityId, ListingType, SortType},
    person::{Login, LoginResponse, Register},
    post::{CreatePost, GetPosts, GetPostsResponse, PostResponse},
    sensitive::Sensitive,
    site::{CreateSite, GetSiteResponse, SiteResponse},
};
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use ureq::{Agent, AgentBuilder};

static LEMMY_BACKEND: &str = "http://localhost:8536";
static LEMMY_API_VERSION: &str = "/api/v3";

pub static AGENT: Lazy<Agent> = Lazy::new(|| {
    AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build()
});

fn gen_request_url(path: &str) -> String {
    format!("{}{}{}", LEMMY_BACKEND, LEMMY_API_VERSION, path)
}

pub fn list_posts() -> Result<GetPostsResponse, Error> {
    let params = GetPosts {
        type_: Some(ListingType::Local),
        sort: Some(SortType::New),
        ..Default::default()
    };
    get("/post/list", Some(params))
}

pub fn create_post(title: &str, auth: Sensitive<String>) -> Result<PostResponse, Error> {
    let params = CreatePost {
        name: title.to_string(),
        community_id: CommunityId(2),
        auth,
        ..Default::default()
    };
    post("/post", &params)
}

pub fn get_site() -> Result<GetSiteResponse, Error> {
    get::<GetSiteResponse, ()>("/site", None)
}

pub fn create_site(auth: Sensitive<String>) -> Result<SiteResponse, Error> {
    let params = CreateSite {
        name: "lemmyBB".to_string(),
        description: Some("Welcome to lemmyBB, enjoy your stay!".to_string()),
        auth,
        ..Default::default()
    };
    post("/site", &params)
}

pub fn register() -> Result<LoginResponse, Error> {
    let pass = Sensitive::new("lemmylemmy".to_string());
    let params = Register {
        username: "lemmy".to_string(),
        password: pass.clone(),
        password_verify: pass,
        ..Default::default()
    };
    post("/user/register", &params)
}

pub fn login() -> Result<LoginResponse, Error> {
    let params = Login {
        username_or_email: Sensitive::new("lemmy".to_string()),
        password: Sensitive::new("lemmylemmy".to_string()),
    };
    post("/user/login", &params)
}

fn post<T, Params>(path: &str, params: Params) -> Result<T, Error>
where
    T: DeserializeOwned,
    Params: Serialize,
{
    Ok(AGENT
        .post(&gen_request_url(path))
        .send_json(&params)?
        .into_json()?)
}

fn get<T, Params>(path: &str, params: Option<Params>) -> Result<T, Error>
where
    T: DeserializeOwned,
    Params: Serialize,
{
    let r = AGENT.get(&gen_request_url(path));
    let r = match params {
        Some(p) => r.send_json(&p)?,
        None => r.call()?,
    };
    Ok(r.into_json()?)
}
