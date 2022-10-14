use crate::api::{get, post};
use anyhow::Error;
use lemmy_api_common::{
    post::{CreatePost, GetPost, GetPostResponse, GetPosts, GetPostsResponse, PostResponse},
    sensitive::Sensitive,
};
use lemmy_db_schema::{
    newtypes::{CommunityId, PostId},
    ListingType,
    SortType,
};

pub async fn list_posts(
    community_id: i32,
    limit: i32,
    page: i32,
    auth: Option<Sensitive<String>>,
) -> Result<GetPostsResponse, Error> {
    let params = GetPosts {
        community_id: Some(CommunityId(community_id)),
        type_: Some(ListingType::Community),
        sort: Some(SortType::NewComments),
        limit: Some(limit.into()),
        page: Some(page.into()),
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
