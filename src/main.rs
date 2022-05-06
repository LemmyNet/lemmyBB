mod api;

use crate::api::{create_post, create_site, get_site, list_posts, login, register};
use anyhow::Error;
use log::{info, LevelFilter};
use maud::{html, Markup, DOCTYPE};
use once_cell::sync::Lazy;
use rouille::{match_assets, router, Response};
use std::time::Duration;
use ureq::{Agent, AgentBuilder};

pub static AGENT: Lazy<Agent> = Lazy::new(|| {
    AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build()
});

fn index() -> Result<Response, Error> {
    Ok(Response::html(html! {
        (DOCTYPE)
        html {
            (header("Hello, world!"))
            @for post in list_posts()?.posts {
                h2 { (post.post.name) }
            }
        }
    }))
}

fn create_test_items() -> Result<(), Error> {
    let site = get_site()?;
    let _auth = if site.site_view.is_none() {
        let auth = register()?.jwt.unwrap();
        create_site(auth.clone())?;
        create_post("test 1", auth.clone())?;
        create_post("test 2", auth.clone())?;
        create_post("test 3", auth.clone())?;
        auth
    } else {
        login()?.jwt.unwrap()
    };
    Ok(())
}

fn header(title: &str) -> Markup {
    html! {
        head {
            title { (title) }
            link href="./styles/prosilver/stylesheet.css" rel="stylesheet";
        }
        h1 { (title) }
    }
}

fn main() {
    env_logger::builder()
        .filter(Some("lemmy_bb"), LevelFilter::Debug)
        .init();

    create_test_items().unwrap();

    info!("Listening on http://127.0.0.1:8080");
    rouille::start_server("127.0.0.1:8080", move |request| {
        if request.url().starts_with("/styles/") {
            return match_assets(request, "assets");
        }

        router!(request,
            (GET) (/) => { index().unwrap() },
            _ => Response::empty_404()
        )
    });
}
