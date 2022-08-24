use crate::api::{
    comment::list_comments,
    post::{get_post, list_posts},
    user::get_person,
};
use anyhow::Error;
use chrono::NaiveDateTime;
use futures::{future::join_all, join};
use lemmy_api_common::{
    comment::GetCommentsResponse,
    post::GetPostsResponse,
    sensitive::Sensitive,
};
use lemmy_db_schema::{
    newtypes::{CommunityId, PostId},
    source::person::PersonSafe,
};
use lemmy_db_views::structs::PostView;
use serde::Serialize;

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
