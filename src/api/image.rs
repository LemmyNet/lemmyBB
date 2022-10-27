use crate::{
    api::{handle_response, CLIENT},
    env::lemmy_backend,
    error::ErrorPage,
    site_fairing::SiteData,
    utils::base_url,
};
use anyhow::{anyhow, Error};
use lemmy_api_common::sensitive::Sensitive;
use rand::distributions::{Alphanumeric, DistString};
use reqwest::multipart::{Form, Part};
use rocket::fs::TempFile;
use serde::Deserialize;
use std::{collections::HashMap, env::temp_dir, fs::File, io::Read};
use url::Url;

/// Pass image requests to Lemmy backend, which forwards it to pictrs
#[get("/pictrs/image/<file>?<params..>")]
pub async fn image(file: String, params: HashMap<String, String>) -> Result<Vec<u8>, ErrorPage> {
    let url = format!("{}/pictrs/image/{}", lemmy_backend(), file);
    let bytes: Vec<u8> = CLIENT
        .get(url)
        .query(&params)
        .send()
        .await?
        .bytes()
        .await?
        .to_vec();
    Ok(bytes)
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
    let res: UploadImageResponse = handle_response(res, &path).await?;
    if res.msg != "ok" {
        return Err(anyhow!(res.msg));
    }
    let url = Url::parse(&format!(
        "{}/pictrs/image/{}",
        base_url(site_data),
        res.files[0].file
    ))?;
    Ok(url)
}
