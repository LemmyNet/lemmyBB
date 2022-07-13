#[macro_use]
extern crate rocket;

mod api;
mod error;
mod routes;
mod template_helpers;
#[cfg(test)]
mod test;

use crate::{
    routes::{
        comment,
        do_comment,
        do_login,
        do_post,
        login_page,
        logout,
        post,
        view_forum,
        view_topic,
    },
    template_helpers::{handlebars_helper_vec_length, markdown, modulo, sum, timestamp},
};
use anyhow::Error;
use log::LevelFilter;
use rocket::{
    fs::{relative, FileServer},
    Build,
    Rocket,
};
use rocket_dyn_templates::Template;
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
        reg.register_helper("sum", Box::new(sum));
        reg.register_helper("mod", Box::new(modulo));
        reg.register_helper("length", Box::new(handlebars_helper_vec_length));
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
