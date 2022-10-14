pub mod comment;
pub mod community;
pub mod post;
pub mod private_message;
pub mod redirects;
pub mod site;
pub mod user;

use crate::{api::CLIENT, error::ErrorPage};
use lemmy_api_common::sensitive::Sensitive;
use rocket::http::CookieJar;

pub fn auth(cookies: &CookieJar<'_>) -> Option<Sensitive<String>> {
    cookies
        .get("jwt")
        .map(|c| Sensitive::new(c.value().to_string()))
}
