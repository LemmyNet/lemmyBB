use anyhow::Error;
use lemmy_api_common::{
    person::{Login, LoginResponse},
    post::{GetPost, GetPostResponse, GetPosts, GetPostsResponse},
    sensitive::Sensitive,
    site::GetSiteResponse,
};
use lemmy_db_schema::{newtypes::PostId, ListingType, SortType};
use once_cell::sync::{Lazy, OnceCell};
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, time::Duration};

static LEMMY_API_VERSION: &str = "/api/v3";

pub static LEMMY_BACKEND: OnceCell<String> = OnceCell::new();
pub static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .expect("build client")
});

fn gen_request_url(path: &str) -> String {
    format!(
        "{}{}{}",
        LEMMY_BACKEND.get().unwrap(),
        LEMMY_API_VERSION,
        path
    )
}

pub async fn list_posts() -> Result<GetPostsResponse, Error> {
    let params = GetPosts {
        type_: Some(ListingType::Local),
        sort: Some(SortType::NewComments),
        limit: Some(20),
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

pub async fn get_site() -> Result<GetSiteResponse, Error> {
    get::<GetSiteResponse, ()>("/site", None).await
}

pub async fn login(username_or_email: &str, password: &str) -> Result<LoginResponse, Error> {
    let params = Login {
        username_or_email: Sensitive::new(username_or_email.to_string()),
        password: Sensitive::new(password.to_string()),
    };
    post("/user/login", &params).await
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
