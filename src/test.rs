use crate::{
    api::{
        categories::CATEGORIES_FILE,
        community::{create_community, delete_community},
        site::create_site,
    },
    init_rocket,
    routes::{community::*, post::*, site::*, user::*},
    site_fairing::test_site_data,
};
use lemmy_api_common::{sensitive::Sensitive, site::GetSiteResponse};
use log::LevelFilter;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::StatusCode;
use rocket::{form::Form, local::asynchronous};
use serial_test::serial;
use std::{env, future::Future, path::Path, time::Duration};
use tokio::{
    task::{spawn_local, LocalSet},
    time::sleep,
};

#[ctor::ctor]
fn init() {
    env::set_var("LEMMYBB_BACKEND", "http://127.0.0.1:8536");
    env_logger::builder().filter_level(LevelFilter::Warn).init();
}

fn random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

async fn run_test<Fut: Future<Output = ()>>(
    test: impl Fn(asynchronous::Client, Sensitive<String>) -> Fut,
) {
    let local = LocalSet::new();
    local
        .run_until(async move {
            let backend = spawn_local(lemmy_server::start_lemmy_server());
            let rocket = init_rocket().unwrap();
            let client = asynchronous::Client::tracked(rocket).await.unwrap();

            // wait for lemmy backend to start and get auth token
            let auth = init_backend().await;

            test(client, auth).await;

            backend.abort();
            let _ = backend.await;
        })
        .await;
}

async fn wait_backend_start() -> Option<GetSiteResponse> {
    for _ in 0..10 {
        let res = reqwest::get("http://127.0.0.1:8536/api/v3/site").await;
        let status = res.as_ref().map(|r| r.status());
        info!("status: {:?}", status);
        if let Ok(StatusCode::OK) = status {
            info!("backend started");
            return res.unwrap().json().await.ok();
        }
        sleep(Duration::from_secs(1)).await;
    }
    None
}

async fn init_backend() -> Sensitive<String> {
    let res: GetSiteResponse = wait_backend_start().await.unwrap();
    let password = random_string();
    let register_form = RegisterForm {
        username: random_string(),
        password: password.clone(),
        password_verify: password,
        ..Default::default()
    };
    // register a new user
    let auth = crate::api::user::register(register_form)
        .await
        .unwrap()
        .jwt
        .unwrap();
    // create site if it doesnt exist yet
    if !res.site_view.local_site.site_setup {
        create_site(
            "test".to_string(),
            Some("test".to_string()),
            false,
            auth.clone(),
        )
        .await
        .unwrap();
    }
    auth
}

#[actix_rt::test]
#[serial]
async fn index() {
    if !Path::new(CATEGORIES_FILE).exists() {
        // write empty categories file so there is no redirect returned (which would have wrong status code)
        std::fs::write("lemmybb_categories.hjson", "[]").unwrap();
    }
    run_test(|client, _auth| async move {
        let res = client.get(uri!("/")).dispatch().await;
        assert_eq!(200, res.status().code);
    })
    .await;
}

#[actix_rt::test]
#[serial]
async fn community_list() {
    run_test(|client, _auth| async move {
        let res = client.get(uri!("/community_list")).dispatch().await;
        assert_eq!(200, res.status().code);
    })
    .await;
}

#[actix_rt::test]
#[serial]
async fn setup() {
    run_test(|client, _auth| async move {
        let res = client.get(uri!(setup)).dispatch().await;
        assert_eq!(200, res.status().code);
    })
    .await;
}

#[actix_rt::test]
#[serial]
async fn view_forum() {
    run_test(|client, auth| async move {
        let created = create_community("my_community".to_string(), auth.clone())
            .await
            .unwrap();
        let f = created.community_view.community.id.0;
        let res = client
            .get(uri!(view_forum(f, None::<i32>, None::<String>)))
            .dispatch()
            .await;
        assert_eq!(200, res.status().code);
        delete_community(created.community_view.community.id, auth.clone())
            .await
            .unwrap();
    })
    .await;
}

#[actix_rt::test]
#[serial]
async fn view_topic() {
    run_test(|_client, auth| async move {
        let community = create_community("my_community_2".to_string(), auth.clone())
            .await
            .unwrap()
            .community_view
            .community;
        let form = PostForm {
            subject: "asd".to_string(),
            message: "dsa".to_string(),
            preview: None,
        };
        let site_data = test_site_data(Some(auth.clone())).await;
        let post = do_post(community.id.0, None, Form::from(form), site_data)
            .await
            .unwrap();
        assert!(post.right().is_some());

        delete_community(community.id, auth).await.unwrap();
    })
    .await;
}

#[actix_rt::test]
#[serial]
async fn login() {
    run_test(|client, _auth| async move {
        let res = client.get(uri!(login)).dispatch().await;
        assert_eq!(200, res.status().code);
    })
    .await;
}

#[actix_rt::test]
#[serial]
async fn register() {
    run_test(|client, _auth| async move {
        let res = client.get(uri!(register)).dispatch().await;
        assert_eq!(200, res.status().code);
    })
    .await;
}

#[actix_rt::test]
#[serial]
async fn search_results() {
    run_test(|client, _auth| async move {
        let res = client
            .get(uri!(search(keywords = "my search")))
            .dispatch()
            .await;
        assert_eq!(200, res.status().code);
    })
    .await;
}

#[actix_rt::test]
#[serial]
async fn view_profile() {
    run_test(|client, _auth| async move {
        let res = client.get(uri!(view_profile(u = 2))).dispatch().await;
        assert_eq!(200, res.status().code);
    })
    .await;
}

#[actix_rt::test]
#[serial]
async fn image() {
    run_test(|client, _auth| async move {
        // TODO: need to upload image before getting it
        let res = client
            .get(uri!(
                "/pictrs/image/24716431-8f92-417a-8492-06d5d3fe9fab.jpeg?thumbnail=120"
            ))
            .dispatch()
            .await;
        dbg!(&res);
        assert_eq!(200, res.status().code);
    })
    .await;
}

#[actix_rt::test]
#[serial]
async fn report() {
    run_test(|client, _auth| async move {
        let res = client.get(uri!("/report?thread=2")).dispatch().await;
        assert_eq!(200, res.status().code);
    })
    .await;
}
