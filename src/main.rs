#[macro_use]
extern crate rocket;

mod api;
mod error;
mod routes;
mod test;

use crate::routes::{
    comment,
    do_comment,
    do_login,
    do_post,
    login_page,
    logout,
    post,
    view_forum,
    view_topic,
};
use anyhow::Error;
use chrono::NaiveDateTime;
use log::LevelFilter;
use rocket::{
    fs::{relative, FileServer},
    Build,
    Rocket,
};
use rocket_dyn_templates::{handlebars::handlebars_helper, Template};
use std::env;

#[main]
async fn main() -> Result<(), Error> {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter(Some("lemmy_bb"), LevelFilter::Debug)
        .filter(Some("rocket"), LevelFilter::Info)
        .init();
    let _ = init_rocket().launch().await?;
    Ok(())
}

fn init_rocket() -> Rocket<Build> {
    let template_fairing = Template::custom(|engines| {
        let reg = &mut engines.handlebars;
        reg.set_strict_mode(true);

        reg.register_helper("markdown", Box::new(markdown));
        reg.register_helper("timestamp", Box::new(timestamp));
    });

    rocket::build()
        .attach(template_fairing)
        .mount(
            "/",
            routes![
                view_forum, view_topic, login_page, do_login, post, do_post, comment, do_comment,
                logout
            ],
        )
        .mount("/assets", FileServer::from(relative!("assets")))
}

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
