mod api;

use crate::api::{create_post, create_site, get_site, list_posts, register};
use anyhow::Error;
use lemmy_db_views::structs::{PostView, SiteView};
use log::{info, LevelFilter};
use once_cell::sync::Lazy;
use rouille::{match_assets, router, Response};
use sailfish::TemplateOnce;
use std::time::Duration;
use ureq::{Agent, AgentBuilder};

pub static AGENT: Lazy<Agent> = Lazy::new(|| {
    AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build()
});

#[derive(TemplateOnce)] // automatically implement `TemplateOnce` trait
#[template(path = "../templates/index.stpl")] // specify the path to template
struct IndexTemplate {
    // data to be passed to the template
    site: SiteView,
    posts: Vec<PostView>,
}

fn index() -> Result<Response, Error> {
    let site = get_site()?.site_view.unwrap();
    let posts = list_posts()?.posts;
    let ctx = IndexTemplate { site, posts };
    Ok(Response::html(ctx.render_once()?))
}

fn create_test_items() -> Result<(), Error> {
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

fn main() {
    env_logger::builder()
        .filter(Some("lemmy_bb"), LevelFilter::Debug)
        .init();

    create_test_items().unwrap();

    info!("Listening on http://127.0.0.1:8080");
    rouille::start_server("127.0.0.1:8080", move |request| {
        if request.url().starts_with("/assets/") {
            return match_assets(request, ".");
        }

        router!(request,
            (GET) (/) => { index().unwrap() },
            _ => Response::empty_404()
        )
    });
}
