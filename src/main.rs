#[macro_use]
extern crate rocket;

mod api;
mod error;
mod routes;

use crate::api::LEMMY_BACKEND;
use anyhow::Error;
use chrono::NaiveDateTime;
use log::{info, LevelFilter};
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::{handlebars::handlebars_helper, Template};
use routes::{do_login, login_page, view_forum, view_topic};
use std::env;

// Converts markdown to html. Use some hacks to change the generated html, so that text size
// and style are consistent with phpBB:
// - remove outer <p> wrapper
// - use <br /><br /> for newlines
// TODO: this currently breaks block quotes and maybe other things
handlebars_helper!(markdown: |md: Option<String>| {
    match md {
    Some(mut o) => {
            o = o.replace("\n\n", "\\\n");
            let mut comrak = comrak::ComrakOptions::default();
            comrak.extension.autolink = true;
            let mut x = comrak::markdown_to_html(&o, &comrak);
            x = x.replace(r"<p>", "");
            x = x.replace(r"</p>", "");
            x = x.replace("<br />", "<br /><br />");
            x
    }
        None => "".to_string()
        }
});

handlebars_helper!(timestamp: |ts: NaiveDateTime| {
    ts.format("%a %h %d, %Y %H:%M").to_string()
});

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
        let reg = &mut engines.handlebars;
        reg.set_strict_mode(true);

        reg.register_helper("markdown", Box::new(markdown));
        reg.register_helper("timestamp", Box::new(timestamp));
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
