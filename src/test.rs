use crate::init_rocket;
use log::LevelFilter;
use rocket::{http::uri::Origin, local::blocking::Client};
use std::env;

#[ctor::ctor]
fn init() {
    env::set_var("LEMMY_INTERNAL_HOST", "https://lemmy.ml");
    env_logger::builder().filter_level(LevelFilter::Warn).init();
}

fn test_with_uri(uri: Origin) {
    let rocket = init_rocket();
    let client = Client::tracked(rocket).unwrap();
    let res = client.get(uri).dispatch();
    assert_eq!(200, res.status().code);
}

#[test]
fn test_viewforum() {
    test_with_uri(uri!("/"))
}

#[test]
fn test_viewtopic() {
    test_with_uri(uri!("/viewtopic?t=360976"))
}

#[test]
fn test_login() {
    test_with_uri(uri!("/login"))
}

#[test]
fn test_post() {
    test_with_uri(uri!("/post"))
}

#[test]
fn test_comment() {
    test_with_uri(uri!("/comment?t=360976"))
}
