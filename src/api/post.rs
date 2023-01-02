use crate::api::{get, post, put};
use anyhow::Error;
use lemmy_api_common::{
    lemmy_db_schema::{
        newtypes::{CommunityId, PostId},
        SortType,
    },
    post::{
        CreatePost,
        CreatePostReport,
        EditPost,
        GetPost,
        GetPostResponse,
        GetPosts,
        GetPostsResponse,
        PostReportResponse,
        PostResponse,
    },
    sensitive::Sensitive,
};

pub async fn list_posts(
    community_id: i32,
    limit: i32,
    page: i32,
    auth: Option<Sensitive<String>>,
) -> Result<GetPostsResponse, Error> {
    let params = GetPosts {
        community_id: Some(CommunityId(community_id)),
        sort: Some(SortType::NewComments),
        limit: Some(limit.into()),
        page: Some(page.into()),
        auth,
        ..Default::default()
    };
    get("/post/list", &params).await
}

pub async fn get_post(id: i32, auth: Option<Sensitive<String>>) -> Result<GetPostResponse, Error> {
    let params = GetPost {
        id: Some(PostId(id)),
        comment_id: None,
        auth,
    };
    get("/post", &params).await
}

pub async fn create_post(
    name: String,
    body: String,
    community_id: i32,
    auth: Sensitive<String>,
) -> Result<PostResponse, Error> {
    let params = CreatePost {
        name,
        body: Some(body),
        community_id: CommunityId(community_id),
        auth,
        ..Default::default()
    };
    post("/post", &params).await
}

pub async fn edit_post(
    name: String,
    body: String,
    post_id: i32,
    auth: Sensitive<String>,
) -> Result<PostResponse, Error> {
    let params = EditPost {
        post_id: PostId(post_id),
        name: Some(name),
        body: Some(body),
        auth,
        ..Default::default()
    };
    put("/post", &params).await
}

pub async fn report_post(
    post_id: i32,
    reason: String,
    auth: Sensitive<String>,
) -> Result<PostReportResponse, Error> {
    let params = CreatePostReport {
        post_id: PostId(post_id),
        reason,
        auth,
    };
    post("/post/report", &params).await
}
