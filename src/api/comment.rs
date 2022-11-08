use crate::api::{get, post, put};
use anyhow::Error;
use lemmy_api_common::{
    comment::{
        CommentReportResponse,
        CommentResponse,
        CreateComment,
        CreateCommentReport,
        EditComment,
        GetComment,
        GetComments,
        GetCommentsResponse,
    },
    sensitive::Sensitive,
};
use lemmy_db_schema::{
    newtypes::{CommentId, CommunityId, PostId},
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
    get("/comment/list", &params).await
}

pub async fn create_comment(
    post_id: i32,
    content: String,
    parent_id: Option<i32>,
    auth: Sensitive<String>,
) -> Result<CommentResponse, Error> {
    let params = CreateComment {
        post_id: PostId(post_id),
        content,
        parent_id: parent_id.map(CommentId),
        auth,
        ..Default::default()
    };
    post("/comment", &params).await
}

pub async fn edit_comment(
    comment_id: i32,
    content: String,
    auth: Sensitive<String>,
) -> Result<CommentResponse, Error> {
    let params = EditComment {
        comment_id: CommentId(comment_id),
        content,
        auth,
        ..Default::default()
    };
    put("/comment", &params).await
}

pub async fn get_comment(
    comment_id: i32,
    auth: Option<Sensitive<String>>,
) -> Result<CommentResponse, Error> {
    let params = GetComment {
        id: CommentId(comment_id),
        auth,
    };
    get("/comment", &params).await
}

pub async fn report_comment(
    comment_id: i32,
    reason: String,
    auth: Sensitive<String>,
) -> Result<CommentReportResponse, Error> {
    let params = CreateCommentReport {
        comment_id: CommentId(comment_id),
        reason,
        auth,
    };
    post("/comment/report", &params).await
}
