use crate::routes::RegisterForm;
use anyhow::{anyhow, Error};
use chrono::NaiveDateTime;
use futures::{future::join_all, join};
use lemmy_api_common::{
    comment::{CommentResponse, CreateComment, GetComments, GetCommentsResponse},
    community::{GetCommunity, GetCommunityResponse, ListCommunities, ListCommunitiesResponse},
    person::{
        GetCaptchaResponse,
        GetPersonDetails,
        GetPersonDetailsResponse,
        Login,
        LoginResponse,
        Register,
    },
    post::{CreatePost, GetPost, GetPostResponse, GetPosts, GetPostsResponse, PostResponse},
    sensitive::Sensitive,
    site::{CreateSite, GetSite, GetSiteResponse, SiteResponse},
};
use lemmy_db_schema::{
    newtypes::{CommunityId, PersonId, PostId},
    source::person::PersonSafe,
    ListingType,
    SortType,
};
use lemmy_db_views::structs::PostView;
use once_cell::sync::Lazy;
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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
    let lemmy_backend =
        env::var("LEMMY_BB_BACKEND").unwrap_or_else(|_| "http://localhost:8536".to_string());

    format!("{}{}{}", lemmy_backend, LEMMY_API_VERSION, path)
}

pub async fn list_posts(
    community_id: i32,
    limit: i64,
    auth: Option<Sensitive<String>>,
) -> Result<GetPostsResponse, Error> {
    let params = GetPosts {
        community_id: Some(CommunityId(community_id)),
        type_: Some(ListingType::Community),
        sort: Some(SortType::NewComments),
        limit: Some(limit),
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
    let mut post: GetPostResponse = get("/post", params).await?;

    // show oldest comments first
    post.comments.sort_unstable_by_key(|a| a.comment.published);

    Ok(post)
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

#[derive(Serialize, Debug)]
pub struct PostOrComment {
    title: String,
    creator: PersonSafe,
    post_id: PostId,
    reply_position: i32,
    time: NaiveDateTime,
}

fn generate_comment_title(post_title: &str) -> String {
    format!("Re: {}", post_title)
}

pub async fn get_last_reply_in_thread(
    post: &PostView,
    auth: Option<Sensitive<String>>,
) -> Result<PostOrComment, Error> {
    if post.counts.comments == 0 {
        Ok(PostOrComment {
            title: post.post.name.clone(),
            creator: post.creator.clone(),
            post_id: post.post.id,
            reply_position: 1,
            time: post.post.published,
        })
    } else {
        let post = get_post(post.post.id.0, auth.clone()).await?;
        let creator_id = post.comments.last().unwrap().comment.creator_id;
        let creator = get_person(creator_id, auth).await?;
        Ok(PostOrComment {
            title: generate_comment_title(&post.post_view.post.name),
            creator: creator.person_view.person,
            post_id: post.post_view.post.id,
            reply_position: (post.comments.len() + 1) as i32,
            time: post.comments.last().unwrap().comment.published,
        })
    }
}

pub async fn get_last_reply_in_community(
    community_id: CommunityId,
    auth: Option<Sensitive<String>>,
) -> Result<Option<PostOrComment>, Error> {
    let (comment, post) = join!(
        list_comments(community_id, auth.clone()),
        list_posts(community_id.0, 1, auth.clone())
    );
    let (comment, post): (GetCommentsResponse, GetPostsResponse) = (comment?, post?);
    let comment = join_all(comment.comments.first().map(|c| async {
        let p = get_post(c.post.id.0, auth).await;
        PostOrComment {
            title: generate_comment_title(&c.post.name),
            creator: c.creator.clone(),
            post_id: c.post.id,
            reply_position: (p.unwrap().post_view.counts.comments + 1) as i32,
            time: c.comment.published,
        }
    }))
    .await
    .pop();
    let post = post.posts.first().map(|p| PostOrComment {
        title: p.post.name.clone(),
        creator: p.creator.clone(),
        post_id: p.post.id,
        reply_position: 1,
        time: p.post.published,
    });
    // return data for post or comment, depending which is newer
    Ok(if let Some(comment) = comment {
        if let Some(post) = post {
            if post.time > comment.time {
                Some(post)
            } else {
                Some(comment)
            }
        } else {
            Some(comment)
        }
    } else {
        None
    })
}

pub async fn list_comments(
    community_id: CommunityId,
    auth: Option<Sensitive<String>>,
) -> Result<GetCommentsResponse, Error> {
    let params = GetComments {
        sort: Some(SortType::NewComments),
        type_: Some(ListingType::Community),
        limit: Some(1),
        community_id: Some(community_id),
        auth,
        ..Default::default()
    };
    get("/comment/list", params).await
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
        limit: Some(10),
        auth,
    };
    get("/community/list", params).await
}

pub async fn get_person(
    person_id: PersonId,
    auth: Option<Sensitive<String>>,
) -> Result<GetPersonDetailsResponse, Error> {
    let params = GetPersonDetails {
        person_id: Some(person_id),
        auth,
        ..Default::default()
    };
    get("/user", params).await
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

pub async fn get_captcha() -> Result<GetCaptchaResponse, Error> {
    get("/user/get_captcha", ()).await
}

pub async fn register(form: RegisterForm) -> Result<LoginResponse, Error> {
    let params = Register {
        username: form.username,
        password: Sensitive::new(form.password),
        password_verify: Sensitive::new(form.password_verify),
        show_nsfw: form.show_nsfw,
        email: form.email.map(Sensitive::new),
        captcha_uuid: form.captcha_uuid,
        captcha_answer: form.captcha_answer,
        honeypot: form.honeypot,
        answer: form.application_answer,
    };
    post("/user/register", &params).await
}

pub async fn create_site(
    name: String,
    description: Option<String>,
    auth: String,
) -> Result<SiteResponse, Error> {
    let params = CreateSite {
        name,
        description,
        auth: Sensitive::new(auth),
        ..Default::default()
    };
    post("/site", &params).await
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
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
        .await?;
    let status = res.status();
    let text = res.text().await?;
    info!("post {} status: {}, response: {}", &path, status, &text);
    handle_response(text, status)
}

async fn get<T, Params>(path: &str, params: Params) -> Result<T, Error>
where
    T: DeserializeOwned,
    Params: Serialize + Debug,
{
    info!("get {}, params {:?}", &path, &params);
    let res = CLIENT
        .get(&gen_request_url(path))
        .query(&params)
        .send()
        .await?;
    let status = res.status();
    let text = res.text().await?;
    info!("get {} status: {}", &path, status);
    handle_response(text, status)
}

fn handle_response<T: DeserializeOwned>(response: String, status: StatusCode) -> Result<T, Error> {
    if status.is_success() {
        Ok(serde_json::from_str(&response)?)
    } else {
        let error: ErrorResponse = serde_json::from_str(&response)?;
        Err(anyhow!(error.error))
    }
}
