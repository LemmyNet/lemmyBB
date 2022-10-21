use crate::{
    api::site::rocket_uri_macro_favicon,
    init_rocket,
    routes::{community::*, post::*, site::*, user::*},
};
use log::LevelFilter;
use rocket::{http::uri::Origin, local::blocking::Client};
use serial_test::serial;
use std::env;

#[ctor::ctor]
fn init() {
    env::set_var("LEMMY_BB_BACKEND", "https://lemmy.ml");
    env_logger::builder().filter_level(LevelFilter::Info).init();
}

fn test_with_uri(uri: Origin) {
    let rocket = init_rocket().unwrap();
    let client = Client::tracked(rocket).unwrap();
    let res = client.get(uri).dispatch();
    assert_eq!(200, res.status().code);
}

#[test]
#[serial]
fn index() {
    test_with_uri(uri!("/"))
}

#[test]
#[serial]
fn community_list() {
    test_with_uri(uri!("/community_list"))
}

#[test]
#[serial]
fn setup() {
    test_with_uri(uri!(setup))
}

#[test]
#[serial]
fn view_forum() {
    test_with_uri(uri!(view_forum(f = 8, page = Some(1))))
}

#[test]
#[serial]
fn view_topic() {
    test_with_uri(uri!(view_topic(t = 360976, page = Some(1))))
}

#[test]
#[serial]
fn login() {
    test_with_uri(uri!(login))
}

#[test]
#[serial]
fn register() {
    test_with_uri(uri!(register))
}

#[test]
#[serial]
fn post_editor() {
    test_with_uri(uri!("/post_editor?f=8"))
}

#[test]
#[serial]
fn comment_editor() {
    test_with_uri(uri!("/comment_editor?t=360976"))
}

#[test]
#[serial]
fn search_results() {
    test_with_uri(uri!(search(keywords = "my search")))
}

#[test]
#[serial]
fn view_profile() {
    test_with_uri(uri!(view_profile(u = 8169)))
}

#[test]
#[serial]
fn favicon() {
    test_with_uri(uri!(favicon))
}

#[test]
#[serial]
fn report() {
    test_with_uri(uri!("/report?thread=2"))
}
