use crate::api::{get, post};
use anyhow::Error;
use lemmy_api_common::{
    comment::{CommentResponse, CreateComment, GetComments, GetCommentsResponse},
    sensitive::Sensitive,
};
use lemmy_db_schema::{
    newtypes::{CommunityId, PostId},
    ListingType,
    SortType,
};

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
