#[macro_use]
extern crate rocket;

mod api;
mod error;
mod routes;
mod template_helpers;
#[cfg(test)]
mod test;

use crate::{
    routes::*,
    template_helpers::{
        comment_index,
        handlebars_helper_vec_length,
        markdown,
        modulo,
        sum,
        timestamp_human,
        timestamp_machine,
    },
};
use anyhow::Error;
use log::LevelFilter;
use rocket::{
    fs::{relative, FileServer},
    Build,
    Config,
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
        .filter(Some("handlebars"), LevelFilter::Info)
        .init();
    let _ = init_rocket()?.launch().await?;
    Ok(())
}

fn init_rocket() -> Result<Rocket<Build>, Error> {
    let template_fairing = Template::custom(|engines| {
        let reg = &mut engines.handlebars;
        reg.set_strict_mode(true);

        reg.register_helper("markdown", Box::new(markdown));
        reg.register_helper("timestamp_human", Box::new(timestamp_human));
        reg.register_helper("timestamp_machine", Box::new(timestamp_machine));
        reg.register_helper("sum", Box::new(sum));
        reg.register_helper("mod", Box::new(modulo));
        reg.register_helper("comment_index", Box::new(comment_index));
        reg.register_helper("length", Box::new(handlebars_helper_vec_length));
    });

    let listen_address =
        env::var("LEMMY_BB_LISTEN_ADDRESS").unwrap_or_else(|_| "127.0.0.1:1244".to_string());
    let (address, port) = listen_address.split_once(':').unwrap();
    let config = Config {
        address: address.parse()?,
        port: port.parse()?,
        ..Config::default()
    };
    Ok(rocket::build()
        .configure(config)
        .attach(template_fairing)
        .mount(
            "/",
            routes![
                index,
                view_forum,
                view_topic,
                login,
                do_login,
                post,
                do_post,
                comment,
                do_comment,
                logout,
                register,
                do_register,
                setup,
                do_setup // TODO: add redirects from apub routes like /post/123
            ],
        )
        .mount("/assets", FileServer::from(relative!("assets"))))
}
