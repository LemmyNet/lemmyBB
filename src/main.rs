#[macro_use]
extern crate rocket;

mod api;
mod error;

use crate::{
    api::{create_post, create_site, get_site, list_posts, register},
    error::ErrorPage,
};
use lemmy_db_views::structs::{PostView, SiteView};
use log::LevelFilter;
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::Template;
use serde::Serialize;

#[derive(Serialize)]
struct IndexTemplate {
    // data to be passed to the template
    site: SiteView,
    posts: Vec<PostView>,
}

#[get("/")]
fn index() -> Result<Template, ErrorPage> {
    let site = get_site()?.site_view.unwrap();
    let posts = list_posts()?.posts;
    let ctx = IndexTemplate { site, posts };
    // TODO: this silently swallows error messages
    Ok(Template::render("index", ctx))
}

fn create_test_items() -> Result<(), ErrorPage> {
    let site = get_site()?;
    if site.site_view.is_none() {
        let auth = register()?.jwt.unwrap();
        create_site(auth.clone())?;
        create_post("test 1", auth.clone())?;
        create_post("test 2", auth.clone())?;
        create_post("test 3", auth)?;
    } else {
        // TODO: this is too slow and blocks startup
        //login()?.jwt.unwrap();
    }
    Ok(())
}

#[main]
async fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter(Some("lemmy_bb"), LevelFilter::Debug)
        .filter(Some("rocket"), LevelFilter::Info)
        .init();

    create_test_items().unwrap();

    info!("Listening on http://127.0.0.1:8000");
    let _ = rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .mount("/assets", FileServer::from(relative!("assets")))
        .launch()
        .await;
}
