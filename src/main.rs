#[macro_use]
extern crate rocket;

mod api;
mod error;
mod routes;

use crate::api::LEMMY_BACKEND;
use anyhow::Error;
use log::{info, LevelFilter};
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::Template;
use routes::{do_login, login_page, view_forum, view_topic};
use std::env;

#[main]
async fn main() -> Result<(), Error> {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter(Some("lemmy_bb"), LevelFilter::Debug)
        .filter(Some("rocket"), LevelFilter::Info)
        .init();

    //create_test_items().await?;

    match env::var("LEMMY_INTERNAL_HOST") {
        Ok(o) => LEMMY_BACKEND.set(o).unwrap(),
        Err(_) => panic!("LEMMY_INTERNAL_HOST environment variable is required"),
    }

    let template_fairing = Template::custom(|engines| {
        engines.handlebars.set_strict_mode(true);
    });

    info!("Listening on http://127.0.0.1:8000");
    let _ = rocket::build()
        .attach(template_fairing)
        .mount("/", routes![view_forum, view_topic, login_page, do_login])
        .mount("/assets", FileServer::from(relative!("assets")))
        .launch()
        .await?;
    Ok(())
}
