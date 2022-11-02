pub mod comment;
pub mod community;
pub mod post;
pub mod private_message;
pub mod redirects;
pub mod site;
pub mod user;

use crate::{api::CLIENT, error::ErrorPage};
use lemmy_api_common::sensitive::Sensitive;
use rocket::http::{Cookie, CookieJar, SameSite};

pub fn auth(cookies: &CookieJar<'_>) -> Option<Sensitive<String>> {
    cookies
        .get("jwt")
        .map(|c| Sensitive::new(c.value().to_string()))
}

pub fn build_jwt_cookie(jwt: Sensitive<String>) -> Cookie<'static> {
    Cookie::build("jwt", jwt.into_inner())
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .finish()
}
