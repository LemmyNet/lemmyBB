use anyhow::Error;
use lemmy_api_common::{
    comment::{CommentResponse, CreateComment},
    community::{GetCommunity, GetCommunityResponse, ListCommunities, ListCommunitiesResponse},
    person::{Login, LoginResponse},
    post::{CreatePost, GetPost, GetPostResponse, GetPosts, GetPostsResponse, PostResponse},
    sensitive::Sensitive,
    site::{GetSite, GetSiteResponse},
};
use lemmy_db_schema::{
    newtypes::{CommunityId, PostId},
    ListingType,
    SortType,
};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::{env, fmt::Debug, time::Duration};

static LEMMY_API_VERSION: &str = "/api/v3";

pub static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .expect("build client")
});

fn gen_request_url(path: &str) -> String {
    let lemmy_backend = env::var("LEMMY_INTERNAL_HOST")
        .unwrap_or_else(|_| panic!("LEMMY_INTERNAL_HOST environment variable is required"));

    format!("{}{}{}", lemmy_backend, LEMMY_API_VERSION, path)
}

pub async fn list_posts(
    community_id: i32,
    auth: Option<Sensitive<String>>,
) -> Result<GetPostsResponse, Error> {
    let params = GetPosts {
        community_id: Some(CommunityId(community_id)),
        type_: Some(ListingType::Community),
        sort: Some(SortType::NewComments),
        limit: Some(20),
        auth,
        ..Default::default()
    };
    get("/post/list", params).await
}

pub async fn get_post(id: i32, auth: Option<Sensitive<String>>) -> Result<GetPostResponse, Error> {
    let params = GetPost {
        id: PostId(id),
        auth,
    };
    get("/post", params).await
}

pub async fn create_post(
    name: String,
    body: String,
    community_id: CommunityId,
    auth: Sensitive<String>,
) -> Result<PostResponse, Error> {
    let params = CreatePost {
        name,
        body: Some(body),
        community_id,
        auth,
        ..Default::default()
    };
    post("/post", params).await
}

pub async fn create_comment(
    post_id: i32,
    content: String,
    auth: Sensitive<String>,
) -> Result<CommentResponse, Error> {
    let params = CreateComment {
        post_id: PostId(post_id),
        content,
        auth,
        ..Default::default()
    };
    post("/comment", params).await
}

pub async fn get_site(auth: Option<Sensitive<String>>) -> Result<GetSiteResponse, Error> {
    let params = GetSite { auth };
    get("/site", params).await
}

pub async fn list_communities(
    auth: Option<Sensitive<String>>,
) -> Result<ListCommunitiesResponse, Error> {
    let params = ListCommunities {
        type_: Some(ListingType::All),
        sort: Some(SortType::TopMonth),
        page: None,
        limit: Some(50),
        auth,
    };
    get("/community/list", params).await
}

pub async fn get_community(
    name: String,
    auth: Option<Sensitive<String>>,
) -> Result<GetCommunityResponse, Error> {
    let params = GetCommunity {
        name: Some(name),
        auth,
        ..Default::default()
    };
    get("/community", params).await
}

pub async fn login(username_or_email: &str, password: &str) -> Result<LoginResponse, Error> {
    let params = Login {
        username_or_email: Sensitive::new(username_or_email.to_string()),
        password: Sensitive::new(password.to_string()),
    };
    post("/user/login", &params).await
}

async fn post<T, Params>(path: &str, params: Params) -> Result<T, Error>
where
    T: DeserializeOwned,
    Params: Serialize + Debug,
{
    info!("post {}, params {:?}", &path, &params);
    let res = CLIENT
        .post(&gen_request_url(path))
        .json(&params)
        .send()
        .await?
        .text()
        .await?;
    info!("post {} response: {}", &path, &res);
    Ok(serde_json::from_str(&res)?)
}

async fn get<T, Params>(path: &str, params: Params) -> Result<T, Error>
where
    T: DeserializeOwned,
    Params: Serialize + Debug,
{
    info!("get {}, params {:?}", &path, &params);
    let r = CLIENT.get(&gen_request_url(path)).query(&params);
    Ok(r.send().await?.json().await?)
}
