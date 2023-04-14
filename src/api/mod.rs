pub mod categories;
pub mod comment;
pub mod community;
pub mod extra;
pub mod image;
pub mod moderation;
pub mod post;
pub mod private_message;
pub mod site;
pub mod user;

use crate::env::lemmy_backend;
use anyhow::{anyhow, Error};
use once_cell::sync::Lazy;
use reqwest::{Client, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};

static LEMMY_API_VERSION: &str = "/api/v3";

pub static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(30))
        .build()
        .expect("build client")
});

pub fn gen_request_url(path: &str) -> String {
    format!("{}{}{}", lemmy_backend(), LEMMY_API_VERSION, path)
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

pub enum NameOrId {
    Name(String),
    Id(i32),
}

async fn post<T, Params>(path: &str, params: &Params) -> Result<T, Error>
where
    T: DeserializeOwned,
    Params: Serialize + Debug,
{
    info!("post {}, params {:?}", &path, &params);
    let res = CLIENT
        .post(&gen_request_url(path))
        .json(&params)
        .send()
        .await?;
    handle_response(res, path).await
}

async fn put<T, Params>(path: &str, params: &Params) -> Result<T, Error>
where
    T: DeserializeOwned,
    Params: Serialize + Debug,
{
    info!("put {}, params {:?}", &path, &params);
    let res = CLIENT
        .put(&gen_request_url(path))
        .json(&params)
        .send()
        .await?;
    handle_response(res, path).await
}

async fn get<T, Params>(path: &str, params: &Params) -> Result<T, Error>
where
    T: DeserializeOwned,
    Params: Serialize + Debug,
{
    info!("get {}, params {:?}", &path, &params);
    let res = CLIENT
        .get(&gen_request_url(path))
        .query(&params)
        .send()
        .await?;
    handle_response(res, path).await
}

pub async fn handle_response<T>(response: Response, path: &str) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let status = response.status();
    info!("{} status: {}", &path, status);
    let text = response.text().await?;
    debug!("Received API response: {}", &text);
    if status.is_success() {
        Ok(json_from_str(&text)?)
    } else {
        let error: ErrorResponse = json_from_str(&text)?;
        Err(anyhow!(error.error))
    }
}

fn json_from_str<'a, T: Deserialize<'a>>(text: &'a str) -> serde_json::Result<T> {
    let res = serde_json::from_str(&text);
    if res.is_err() {
        warn!("Failed to deserialize API response: {text}");
    }
    res
}
