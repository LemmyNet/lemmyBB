use anyhow::Error;
use lemmy_api_common::post::{GetPosts, GetPostsResponse};
use lemmy_db_schema::{ListingType, SortType};
use log::{info, LevelFilter};
use maud::{html, Markup, DOCTYPE};
use once_cell::sync::Lazy;
use rouille::{match_assets, router, Response};
use std::time::Duration;
use ureq::{Agent, AgentBuilder};

static AGENT: Lazy<Agent> = Lazy::new(|| {
    AgentBuilder::new()
        .timeout_read(Duration::from_secs(5))
        .timeout_write(Duration::from_secs(5))
        .build()
});

fn index() -> Result<Response, Error> {
    let params = GetPosts {
        type_: Some(ListingType::Local),
        sort: Some(SortType::New),
        ..Default::default()
    };
    let posts = AGENT
        .get("http://localhost:8536/api/v3/post/list")
        .send_json(&params)?
        .into_json::<GetPostsResponse>()?;

    Ok(Response::html(html! {
        (DOCTYPE)
        html {
            (header("Hello, world!"))
            @for post in posts.posts {
                h2 { (post.post.name) }
            }
        }
    }))
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
        .filter(None, LevelFilter::Debug)
        .init();
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
