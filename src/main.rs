#[macro_use]
extern crate rocket;

mod api;
mod error;

use crate::{
    api::{create_site, get_site, list_posts, login, register, resolve_object},
    error::ErrorPage,
};
use anyhow::Error;
use lemmy_api_common::lemmy_db_views::structs::{PostView, SiteView};
use log::{info, LevelFilter};
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::Template;
use serde::Serialize;
use std::thread::spawn;

#[derive(Serialize)]
struct IndexTemplate {
    // data to be passed to the template
    site: SiteView,
    posts: Vec<PostView>,
}

#[get("/")]
async fn index() -> Result<Template, ErrorPage> {
    let site = get_site().await?.site_view.unwrap();
    let posts = list_posts().await?.posts;
    let ctx = IndexTemplate { site, posts };
    Ok(Template::render("index", ctx))
}

async fn create_test_items() -> Result<(), Error> {
    //TODO: these usually fail with timeout, as http_fetch_retry_limit is reached
    resolve_object("https://lemmy.ml/c/opensource".to_string())
        .await
        .ok();
    resolve_object("https://lemmy.ml/c/announcements".to_string())
        .await
        .ok();
    resolve_object("https://lemmy.ml/c/asklemmy".to_string())
        .await
        .ok();

    let site = get_site().await?;
    let _jwt = if site.site_view.is_none() {
        let auth = register().await?.jwt.unwrap();
        create_site(auth.clone()).await?;
        auth
    } else {
        login().await?.jwt.unwrap()
    };
    Ok(())
}

#[main]
async fn main() -> Result<(), Error> {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter(Some("lemmy_bb"), LevelFilter::Debug)
        .filter(Some("rocket"), LevelFilter::Info)
        .init();

    spawn(|| create_test_items());

    info!("Listening on http://127.0.0.1:8000");
    let _ = rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .mount("/assets", FileServer::from(relative!("assets")))
        .launch()
        .await?;
    Ok(())
}
