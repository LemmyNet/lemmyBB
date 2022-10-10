use crate::api::get;
use anyhow::Error;
use lemmy_api_common::{
    community::{GetCommunity, GetCommunityResponse, ListCommunities, ListCommunitiesResponse},
    sensitive::Sensitive,
};
use lemmy_db_schema::{newtypes::CommunityId, ListingType, SortType};

pub async fn list_communities(
    page: Option<i64>,
    auth: Option<Sensitive<String>>,
) -> Result<ListCommunitiesResponse, Error> {
    let params = ListCommunities {
        type_: Some(ListingType::All),
        sort: Some(SortType::TopMonth),
        page,
        limit: Some(20),
        auth,
    };
    get("/community/list", params).await
}

pub async fn get_community(
    id: i32,
    auth: Option<Sensitive<String>>,
) -> Result<GetCommunityResponse, Error> {
    let params = GetCommunity {
        id: Some(CommunityId(id)),
        auth,
        ..Default::default()
    };
    get("/community", params).await
}
