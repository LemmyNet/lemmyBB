use crate::{
    api::{
        comment::{list_comments, list_community_comments},
        post::{get_post, list_posts},
        user::{get_person, list_mentions, list_replies},
        NameOrId,
    },
    env::increased_rate_limit,
};
use anyhow::Error;
use chrono::NaiveDateTime;
use futures::future::{join, join_all};
use lemmy_api_common::{
    comment::GetCommentsResponse,
    lemmy_db_schema::{
        newtypes::{CommunityId, PostId},
        source::person::PersonSafe,
    },
    lemmy_db_views::structs::{CommentView, PostView},
    post::GetPostsResponse,
    sensitive::Sensitive,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct PostOrComment {
    title: String,
    creator: PersonSafe,
    post_id: PostId,
    reply_id: i32,
    time: NaiveDateTime,
}

fn generate_comment_title(post_title: &str) -> String {
    format!("Re: {post_title}")
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
            reply_id: post.post.id.0,
            time: post.post.published,
        })
    } else {
        let post = get_post(post.post.id.0, auth.clone()).await?;
        let comments: Vec<CommentView> = list_comments(post.post_view.post.id, auth.clone())
            .await?
            .into_iter()
            .filter(|c| !c.comment.deleted && !c.comment.removed)
            .collect();
        let creator_id = comments.last().unwrap().comment.creator_id;
        let creator = get_person(NameOrId::Id(creator_id.0), auth).await?;
        let last_comment = &comments.last().unwrap().comment;
        Ok(PostOrComment {
            title: generate_comment_title(&post.post_view.post.name),
            creator: creator.person_view.person,
            post_id: post.post_view.post.id,
            reply_id: last_comment.id.0,
            time: last_comment.published,
        })
    }
}

pub async fn get_last_reply_in_community(
    community_id: CommunityId,
    auth: Option<Sensitive<String>>,
) -> Result<Option<PostOrComment>, Error> {
    if !increased_rate_limit() {
        return Ok(None);
    }
    let (comment, post) = join(
        list_community_comments(community_id, auth.clone()),
        list_posts(community_id.0, 1, 1, auth.clone()),
    )
    .await;
    let (comment, post): (GetCommentsResponse, GetPostsResponse) = (comment?, post?);
    let comment = join_all(
        comment
            .comments
            .iter()
            .filter(|c| !c.comment.deleted && !c.comment.removed)
            .last()
            .map(|c| async {
                PostOrComment {
                    title: generate_comment_title(&c.post.name),
                    creator: c.creator.clone(),
                    post_id: c.post.id,
                    reply_id: c.comment.id.0,
                    time: c.comment.published,
                }
            }),
    )
    .await
    .pop();
    let post = post
        .posts
        .iter()
        .filter(|p| !p.post.deleted && !p.post.removed)
        .last()
        .map(|p| PostOrComment {
            title: p.post.name.clone(),
            creator: p.creator.clone(),
            post_id: p.post.id,
            reply_id: p.post.id.0,
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
        post
    })
}

#[derive(Serialize, Debug, Deserialize, Clone)]
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
            link: format!("/view_topic?t={}", m.post.id),
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
            link: format!("/view_topic?t={}", r.post.id),
        })
        .collect();
    let mut notifications = mentions;
    notifications.append(&mut replies);
    notifications.sort_by_key(|n| n.time);
    Ok(notifications)
}
