mod api;

use crate::api::{create_post, create_site, get_site, list_posts, register};
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
            (html_header("Hello, world!"))
            body id="phpbb" class="nojs notouch section-index ltr" {
                div id="wrap" class="wrap" {
                    a id="top" class="top-anchor" accesskey="t" {}
                    (page_header())
                    @for post in list_posts()?.posts {
                        h2 { (post.post.name) }
                    }
                }
            }
        }
    }))
}

fn html_header(title: &str) -> Markup {
    html! {
        head {
            title { (title) }
            link href="/assets/css/font-awesome.min.css" rel="stylesheet";
            link href="/assets/styles/prosilver/stylesheet.css" rel="stylesheet";
        }
    }
}

fn page_header() -> Markup {
    html! {
        div id="page-header" {
            div class="headerbar" role="banner" {
                div class="inner" {
                    div id="site-description" class="site-description" {
                        a id="logo" class="logo" href="/" title="Board index" {
                            span class="site_logo" {}
                        }
                        h1 { "yourdomain.com" }
                        p { "Default username/password: lemmy/lemmylemmy" }
                        p class="skiplink" { a href="#start_here" { "Skip to content"} }
                    }
                    div id="search-box" class="search-box search-header" role="search" {
                        form action="./search.php" method="get" id="search" {
                            fieldset {
                                input name="keywords" id="keywords" type="search" maxlength="128" title="Search for keywords" class="inputbox search tiny" size="20" value="" placeholder="Search…";
                                button class="button button-search" type="submit" title="Search" {
                                i class="icon fa-search fa-fw" aria-hidden="true" {}
                                span class="sr-only" { "Search" }
                                }
                                a href="./search.php" class="button button-search-end" title="Advanced search" {
                                i class="icon fa-cog fa-fw" aria-hidden="true" {}
                                span class="sr-only" { "Advanced search" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
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
