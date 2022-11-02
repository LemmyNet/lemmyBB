use crate::api::{get, post};
use anyhow::Error;
use futures::future::join;
use lemmy_api_common::{
    sensitive::Sensitive,
    site::{
        CreateSite,
        ResolveObject,
        ResolveObjectResponse,
        Search,
        SearchResponse,
        SiteResponse,
    },
};

pub async fn create_site(
    name: String,
    description: Option<String>,
    auth: Sensitive<String>,
) -> Result<SiteResponse, Error> {
    let params = CreateSite {
        name,
        description,
        auth,
        ..Default::default()
    };
    post("/site", &params).await
}

pub async fn search(
    query: String,
    auth: Option<Sensitive<String>>,
) -> Result<SearchResponse, Error> {
    let search_params = Search {
        q: query.clone(),
        auth: auth.clone(),
        ..Default::default()
    };
    let (search, resolve) = join(get("/search", &search_params), resolve_object(query, auth)).await;

    // ignore resolve errors, those will happen every time that query is not an apub id
    let (mut search, resolve): (SearchResponse, ResolveObjectResponse) =
        (search?, resolve.unwrap_or_default());
    // for simplicity, we just append result from resolve_object to search result
    if let Some(p) = resolve.post {
        search.posts.push(p)
    };
    if let Some(c) = resolve.comment {
        search.comments.push(c)
    };
    if let Some(c) = resolve.community {
        search.communities.push(c)
    };
    if let Some(p) = resolve.person {
        search.users.push(p)
    };
    Ok(search)
}

pub async fn resolve_object(
    query: String,
    auth: Option<Sensitive<String>>,
) -> Result<ResolveObjectResponse, Error> {
    let resolve_params = ResolveObject { q: query, auth };
    match get("/resolve_object", &resolve_params).await {
        Err(e) => {
            warn!("Failed to resolve object {}: {}", resolve_params.q, e);
            Err(e)
        }
        o => o,
    }
}
