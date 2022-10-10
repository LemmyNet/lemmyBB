pub mod comment;
pub mod community;
pub mod extra;
pub mod post;
pub mod private_message;
pub mod site;
pub mod user;

use crate::env::lemmy_backend;
use anyhow::{anyhow, Error};
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
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

fn gen_request_url(path: &str) -> String {
    format!("{}{}{}", lemmy_backend(), LEMMY_API_VERSION, path)
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
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
        .await?;
    let status = res.status();
    let text = res.text().await?;
    info!("post {} status: {}, response: {}", &path, status, &text);
    handle_response(text, status)
}

async fn get<T, Params>(path: &str, params: Params) -> Result<T, Error>
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
    let status = res.status();
    let text = res.text().await?;
    info!("get {} status: {}", &path, status);
    handle_response(text, status)
}

fn handle_response<T: DeserializeOwned>(response: String, status: StatusCode) -> Result<T, Error> {
    if status.is_success() {
        Ok(serde_json::from_str(&response)?)
    } else {
        let error: ErrorResponse = serde_json::from_str(&response)?;
        Err(anyhow!(error.error))
    }
}
