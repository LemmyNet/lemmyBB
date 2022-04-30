use actix_web::{get, App, HttpServer, Result as AwResult};
use log::{info, LevelFilter};
use maud::{html, Markup};

#[get("/")]
async fn index() -> AwResult<Markup> {
    Ok(html! {
        h1 { "Hello, world!" }
        p.intro {
            "This is an example of the "
            a href="https://github.com/lambda-fairy/maud" { "Maud" }
            " template language."
        }
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().filter(None, LevelFilter::Info).init();
    info!("Listening on http://127.0.0.1:8080");
    HttpServer::new(|| App::new().service(index))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
