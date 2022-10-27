#[macro_use]
extern crate rocket;
#[macro_use]
extern crate json_gettext;

mod api;
mod env;
mod error;
mod pagination;
mod routes;
mod site_fairing;
mod template_helpers;
#[cfg(test)]
mod test;
mod utils;

use crate::{
    api::image::image,
    env::listen_address,
    routes::{
        comment::*,
        community::*,
        post::*,
        private_message::*,
        redirects::*,
        site::*,
        user::*,
    },
    site_fairing::SiteFairing,
    template_helpers::*,
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
        reg.register_helper("add", Box::new(add));
        reg.register_helper("sub", Box::new(sub));
        reg.register_helper("mod", Box::new(modulo));
        reg.register_helper("comment_page", Box::new(comment_page));
        reg.register_helper("length", Box::new(length));
        reg.register_helper("community_actor_id", Box::new(community_actor_id));
        reg.register_helper("user_actor_id", Box::new(user_actor_id));
        reg.register_helper("user_name", Box::new(user_name));
        reg.register_helper("concat", Box::new(concat));
        reg.register_helper("i18n", Box::new(i18n));
    });

    let listen_address = listen_address();
    let (address, port) = listen_address.split_once(':').unwrap();
    let config = Config {
        address: address.parse()?,
        port: port.parse()?,
        ..Config::default()
    };
    Ok(rocket::build()
        .configure(config)
        .attach(template_fairing)
        .attach(SiteFairing {})
        .mount(
            "/",
            routes![
                index,
                view_forum,
                view_topic,
                login,
                do_login,
                post_editor,
                do_post,
                comment_editor,
                do_comment,
                logout,
                register,
                do_register,
                setup,
                do_setup,
                mark_all_notifications_read,
                legal,
                search,
                view_profile,
                private_messages_list,
                private_messages_thread,
                private_message_editor,
                do_send_private_message,
                image,
                redirect_apub_community,
                redirect_apub_user,
                redirect_apub_post,
                redirect_apub_comment,
                report,
                do_report,
                edit_profile,
                do_edit_profile,
                community_list
            ],
        )
        .mount("/assets", FileServer::from(relative!("assets"))))
}
