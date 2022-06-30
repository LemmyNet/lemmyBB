#[macro_use]
extern crate rocket;

mod api;
mod error;
mod routes;

use crate::api::{create_site, get_site, register, resolve_object};
use anyhow::Error;
use log::{info, LevelFilter};
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::Template;
use routes::{do_login, login_page, view_forum, view_topic};

async fn create_test_items() -> Result<(), Error> {
    //TODO: these usually fail with timeout, as http_fetch_retry_limit is reached
    resolve_object("https://lemmy.ca/comment/95619".to_string())
        .await
        .ok();
    resolve_object("https://lemmy.ml/c/announcements".to_string())
        .await
        .ok();
    resolve_object("https://lemmy.ml/c/asklemmy".to_string())
        .await
        .ok();

    let site = get_site().await?;
    if site.site_view.is_none() {
        let auth = register().await?.jwt.unwrap();
        create_site(auth.clone()).await?;
    }
    Ok(())
}

#[main]
async fn main() -> Result<(), Error> {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter(Some("lemmy_bb"), LevelFilter::Debug)
        .filter(Some("rocket"), LevelFilter::Info)
        .init();

    create_test_items().await?;

    info!("Listening on http://127.0.0.1:8000");
    let _ = rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![view_forum, view_topic, login_page, do_login])
        .mount("/assets", FileServer::from(relative!("assets")))
        .launch()
        .await?;
    Ok(())
}
