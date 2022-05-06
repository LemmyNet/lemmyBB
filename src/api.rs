use crate::AGENT;
use anyhow::Error;
use lemmy_api_common::{
    person::{Login, LoginResponse, Register},
    post::{CreatePost, GetPosts, GetPostsResponse, PostResponse},
    sensitive::Sensitive,
    site::{CreateSite, GetSiteResponse, SiteResponse},
};
use lemmy_db_schema::{newtypes::CommunityId, ListingType, SortType};

static LEMMY_BACKEND: &str = "http://localhost:8536";
static LEMMY_API_VERSION: &str = "/api/v3";

fn gen_request_url(path: &str) -> String {
    format!("{}{}{}", LEMMY_BACKEND, LEMMY_API_VERSION, path)
}

pub fn list_posts() -> Result<GetPostsResponse, Error> {
    let params = GetPosts {
        type_: Some(ListingType::Local),
        sort: Some(SortType::New),
        ..Default::default()
    };
    Ok(AGENT
        .get(&gen_request_url("/post/list"))
        .send_json(&params)?
        .into_json()?)
}

pub fn create_post(title: &str, auth: Sensitive<String>) -> Result<PostResponse, Error> {
    let create = CreatePost {
        name: title.to_string(),
        community_id: CommunityId(2),
        auth,
        ..Default::default()
    };
    Ok(AGENT
        .post(&gen_request_url("/post"))
        .send_json(&create)?
        .into_json()?)
}

pub fn get_site() -> Result<GetSiteResponse, Error> {
    Ok(AGENT.get(&gen_request_url("/site")).call()?.into_json()?)
}

pub fn create_site(auth: Sensitive<String>) -> Result<SiteResponse, Error> {
    let params = CreateSite {
        name: "lemmy".to_string(),
        auth,
        ..Default::default()
    };
    dbg!(&params);
    Ok(AGENT
        .post(&gen_request_url("/site"))
        .send_json(&params)?
        .into_json()?)
}

pub fn register() -> Result<LoginResponse, Error> {
    let pass = Sensitive::new("lemmylemmy".to_string());
    let params = Register {
        username: "lemmy".to_string(),
        password: pass.clone(),
        password_verify: pass,
        ..Default::default()
    };
    Ok(AGENT
        .post(&gen_request_url("/user/register"))
        .send_json(&params)?
        .into_json()?)
}

pub fn login() -> Result<LoginResponse, Error> {
    let params = Login {
        username_or_email: Sensitive::new("lemmy".to_string()),
        password: Sensitive::new("lemmylemmy".to_string()),
    };
    Ok(AGENT
        .post(&gen_request_url("/user/login"))
        .send_json(&params)?
        .into_json()?)
}
