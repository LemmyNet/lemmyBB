use crate::{
    api::{get, post, NameOrId},
    pagination::PAGE_ITEMS,
};
use anyhow::Error;
use lemmy_api_common::{
    community::{
        CommunityResponse,
        FollowCommunity,
        GetCommunity,
        GetCommunityResponse,
        ListCommunities,
        ListCommunitiesResponse,
    },
    sensitive::Sensitive,
};
use lemmy_db_schema::{newtypes::CommunityId, ListingType, SortType};

pub async fn list_communities(
    listing_type: ListingType,
    page: Option<i32>,
    auth: Option<Sensitive<String>>,
) -> Result<ListCommunitiesResponse, Error> {
    let params = ListCommunities {
        type_: Some(listing_type),
        sort: Some(SortType::NewComments),
        page: page.map(Into::into),
        limit: Some(PAGE_ITEMS.into()),
        auth,
    };
    get("/community/list", params).await
}

pub async fn get_community(
    name_or_id: NameOrId,
    auth: Option<Sensitive<String>>,
) -> Result<GetCommunityResponse, Error> {
    let mut params = GetCommunity {
        auth,
        ..Default::default()
    };
    match name_or_id {
        NameOrId::Name(n) => params.name = Some(n),
        NameOrId::Id(c) => params.id = Some(CommunityId(c)),
    }
    get("/community", params).await
}

pub async fn follow_community(
    community_id: i32,
    follow: bool,
    auth: Sensitive<String>,
) -> Result<CommunityResponse, Error> {
    let params = FollowCommunity {
        community_id: CommunityId(community_id),
        follow,
        auth,
    };
    post("/community/follow", params).await
}
