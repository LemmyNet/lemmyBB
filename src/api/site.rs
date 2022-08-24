use crate::api::{get, post};
use anyhow::Error;
use lemmy_api_common::{
    sensitive::Sensitive,
    site::{CreateSite, GetSite, GetSiteResponse, SiteResponse},
};

pub async fn get_site(auth: Option<Sensitive<String>>) -> Result<GetSiteResponse, Error> {
    let params = GetSite { auth };
    get("/site", params).await
}

pub async fn create_site(
    name: String,
    description: Option<String>,
    auth: String,
) -> Result<SiteResponse, Error> {
    let params = CreateSite {
        name,
        description,
        auth: Sensitive::new(auth),
        ..Default::default()
    };
    post("/site", &params).await
}
