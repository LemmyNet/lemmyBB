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
        backend_endpoints::*,
        comment::*,
        community::*,
        moderation::*,
        post::*,
        private_message::*,
        site::*,
        user::*,
    },
    site_fairing::SiteFairing,
    template_helpers::*,
};
use anyhow::Error;
use env_logger::Env;
use rocket::{
    fs::{relative, FileServer},
    Build,
    Config,
    Rocket,
};
use rocket_dyn_templates::Template;

#[main]
async fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(
        Env::default().default_filter_or("warn,lemmy_bb=debug,handlebars=info"),
    )
    .init();
    let rocket = init_rocket()?.launch();
    #[cfg(not(feature = "embed-lemmy"))]
    let _ = rocket.await?;
    #[cfg(feature = "embed-lemmy")]
    {
        let lemmy = send_wrapper::SendWrapper::new(lemmy_server::start_lemmy_server());
        let (frontend, backend) = tokio::join!(rocket, lemmy);
        let _ = frontend.unwrap();
        if let Err(e) = backend {
            log::error!("{}", e);
        }
    }
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
        reg.register_helper("raw", Box::new(raw));
        reg.register_helper("is_mod", Box::new(is_mod));
        reg.register_helper("is_mod_or_admin", Box::new(is_mod_or_admin));
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
                apub_community,
                apub_user,
                apub_post,
                apub_comment,
                report,
                do_report,
                edit_profile,
                do_edit_profile,
                community_list,
                inboxes,
                feeds,
                well_known,
                node_info,
                api_site,
                remove_item,
                do_remove_item,
                mod_log
            ],
        )
        .mount("/assets", FileServer::from(relative!("assets"))))
}
