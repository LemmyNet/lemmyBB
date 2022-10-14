use crate::api::{
    comment::list_comments,
    post::{get_post, list_posts},
    user::{get_person, list_mentions, list_replies},
    NameOrId,
};
use anyhow::Error;
use chrono::NaiveDateTime;
use futures::future::{join, join_all};
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

#[allow(dead_code)]
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
        let creator = get_person(NameOrId::Id(creator_id.0), auth).await?;
        Ok(PostOrComment {
            title: generate_comment_title(&post.post_view.post.name),
            creator: creator.person_view.person,
            post_id: post.post_view.post.id,
            reply_position: (post.comments.len() + 1) as i32,
            time: post.comments.last().unwrap().comment.published,
        })
    }
}

#[allow(dead_code)]
pub async fn get_last_reply_in_community(
    community_id: CommunityId,
    auth: Option<Sensitive<String>>,
) -> Result<Option<PostOrComment>, Error> {
    let (comment, post) = join(
        list_comments(community_id, auth.clone()),
        list_posts(community_id.0, 1, 1, auth.clone()),
    )
    .await;
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

#[derive(Serialize, Debug)]
pub struct Notification {
    pub title: String,
    pub from_user: PersonSafe,
    pub reference: String,
    pub time: NaiveDateTime,
    pub link: String,
}

/// combine all types of notifications in a single "api call"
pub async fn get_notifications(auth: Sensitive<String>) -> Result<Vec<Notification>, Error> {
    let (m, r) = join(list_mentions(auth.clone()), list_replies(auth.clone())).await;
    // TODO: would be good if we can find out the comment's position in the topic, and link like
    //       viewtopic?t=1#p2
    let mentions: Vec<Notification> = m?
        .mentions
        .into_iter()
        .map(|m| Notification {
            title: "Mention".to_string(),
            from_user: m.creator,
            reference: m.comment.content,
            time: m.comment.published,
            link: format!("/viewtopic?t={}", m.post.id),
        })
        .collect();
    let mut replies = r?
        .replies
        .into_iter()
        .map(|r| Notification {
            title: "Reply".to_string(),
            from_user: r.creator,
            reference: r.comment.content,
            time: r.comment.published,
            link: format!("/viewtopic?t={}", r.post.id),
        })
        .collect();
    let mut notifications = mentions;
    notifications.append(&mut replies);
    notifications.sort_by_key(|n| n.time);
    Ok(notifications)
}
