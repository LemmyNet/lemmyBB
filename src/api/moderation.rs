use crate::api::{get, post};
use anyhow::Error;
use lemmy_api_common::{
    comment::{CommentResponse, RemoveComment},
    lemmy_db_schema::newtypes::{CommentId, PostId},
    post::{PostResponse, RemovePost},
    sensitive::Sensitive,
    site::{GetModlog, GetModlogResponse},
};

pub async fn remove_post(
    post_id: i32,
    reason: String,
    auth: Sensitive<String>,
) -> Result<PostResponse, Error> {
    let params = RemovePost {
        post_id: PostId(post_id),
        removed: true,
        reason: Some(reason),
        auth,
    };
    post("/post/remove", &params).await
}

pub async fn remove_comment(
    comment_id: i32,
    reason: String,
    auth: Sensitive<String>,
) -> Result<CommentResponse, Error> {
    let params = RemoveComment {
        comment_id: CommentId(comment_id),
        removed: true,
        reason: Some(reason),
        auth,
    };
    post("/comment/remove", &params).await
}

pub async fn get_mod_log(auth: Option<Sensitive<String>>) -> Result<GetModlogResponse, Error> {
    let params = GetModlog {
        auth,
        ..Default::default()
    };
    get("/modlog", &params).await
}
