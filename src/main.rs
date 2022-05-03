use actix_files::Files;
use actix_web::{get, App, HttpServer, Result};
use lemmy_api_common::post::{GetPosts, GetPostsResponse};
use lemmy_db_schema::{ListingType, SortType};
use log::{info, LevelFilter};
use maud::{html, Markup, DOCTYPE};
use once_cell::sync::Lazy;
use reqwest::Client;

static CLIENT: Lazy<Client> = Lazy::new(Client::new);

#[get("/")]
async fn index() -> Result<Markup> {
    let params = GetPosts {
        type_: Some(ListingType::Local),
        sort: Some(SortType::New),
        ..Default::default()
    };
    let posts = CLIENT
        .get("https://lemmy.ml/api/v3/post/list")
        .json(&params)
        .send()
        .await
        .unwrap()
        .json::<GetPostsResponse>()
        .await
        .unwrap();

    Ok(html! {
        (DOCTYPE)
        html {
            (header("Hello, world!"))
            @for post in posts.posts {
                h2 { (post.post.name) }
            }
        }
    })
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().filter(None, LevelFilter::Info).init();
    info!("Listening on http://127.0.0.1:8080");
    HttpServer::new(|| App::new()
      .service(index)
      .service(Files::new("/styles", "assets/styles"))
    )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
