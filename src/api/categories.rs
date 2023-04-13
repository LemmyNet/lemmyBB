use crate::api::{
    extra::{get_last_reply_in_community, PostOrComment},
    site::resolve_object,
};
use anyhow::Error;
use deser_hjson::from_str;
use futures::future::join_all;
use lemmy_api_common::{
    lemmy_db_views_actor::structs::CommunityView,
    sensitive::Sensitive,
    site::ResolveObjectResponse,
};
use std::fs::read_to_string;

pub static CATEGORIES_FILE: &str = "lemmybb_categories.hjson";

type CategoriesConfig = Vec<(String, Vec<String>)>;
type CategoriesConfigParsed = Vec<(String, Vec<(CommunityView, Option<PostOrComment>)>)>;

fn read_categories_file() -> Result<CategoriesConfig, Error> {
    let config_str = read_to_string(CATEGORIES_FILE)?;
    Ok(from_str::<CategoriesConfig>(&config_str)?)
}

pub async fn get_categories(
    auth: Option<Sensitive<String>>,
) -> Result<CategoriesConfigParsed, Error> {
    let categories_config = read_categories_file()?;

    let mut resolved = CategoriesConfigParsed::new();
    for (key, communities) in categories_config {
        // first resolve communities
        // TODO: this only needs to be done once on startup, then store CommunityId
        let communities = join_all(
            communities
                .iter()
                .map(|c| resolve_object(c.to_string(), auth.clone())),
        )
        .await;
        // handle errors
        let communities = communities
            .into_iter()
            .collect::<Result<Vec<ResolveObjectResponse>, Error>>()?;
        // extract community from response
        let communities: Vec<CommunityView> = communities
            .into_iter()
            .map(|r| r.community.unwrap())
            .collect();
        // fetch last replies in communities
        let last_replies = join_all(
            communities
                .iter()
                .map(|c| get_last_reply_in_community(c.community.id, auth.clone())),
        )
        .await;
        // handle errors
        let last_replies = last_replies
            .into_iter()
            .collect::<Result<Vec<Option<PostOrComment>>, Error>>()?;
        // merge collections
        let zipped = communities
            .into_iter()
            .zip(last_replies.into_iter())
            .collect();
        resolved.push((key, zipped));
    }

    Ok(resolved)
}
