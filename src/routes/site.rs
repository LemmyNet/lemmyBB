use crate::{
    api::{
        categories::get_categories,
        community::list_communities,
        extra::{get_last_reply_in_community, PostOrComment},
        site::create_site,
        user::register,
    },
    pagination::{PageLimit, Pagination},
    routes::{user::RegisterForm, ErrorPage},
    site_fairing::SiteData,
};
use anyhow::Error;
use futures::future::join_all;
use lemmy_db_schema::ListingType;
use lemmy_db_views_actor::structs::CommunityView;
use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    response::Redirect,
    Either,
};
use rocket_dyn_templates::{context, Template};
use std::str::FromStr;

#[get("/")]
pub async fn index(site_data: SiteData) -> Result<Either<Redirect, Template>, ErrorPage> {
    if site_data.site.site_view.is_none() {
        // need to setup site
        return Ok(Either::Left(Redirect::to(uri!(setup))));
    }

    match get_categories(site_data.auth.clone()).await {
        Ok(categories) => {
            let ctx = context! { site_data, categories };
            Ok(Either::Right(Template::render("site/index", ctx)))
        }
        Err(e) => {
            warn!("{}", e);
            Ok(Either::Left(Redirect::to(uri!("/community_list"))))
        }
    }
}

#[get("/community_list?<mode>&<page>")]
pub async fn community_list(
    page: Option<i32>,
    mode: Option<&str>,
    site_data: SiteData,
) -> Result<Either<Redirect, Template>, ErrorPage> {
    let auth = site_data.auth.clone();
    let listing_type: ListingType = mode
        .map(ListingType::from_str)
        .unwrap_or(Ok(ListingType::All))?;
    let mut communities: Vec<CommunityView> = list_communities(listing_type, page, auth.clone())
        .await?
        .communities;
    communities.sort_unstable_by_key(|c| c.community.id.0);
    let last_replies = join_all(
        communities
            .iter()
            .map(|c| get_last_reply_in_community(c.community.id, auth.clone())),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<Option<PostOrComment>>, Error>>()?;

    let limit = PageLimit::Unknown(communities.len());
    let pagination = Pagination::new(page.unwrap_or(1), limit, "/community_list??");
    let ctx = context! { site_data, communities, last_replies, pagination };
    Ok(Either::Right(Template::render("site/community_list", ctx)))
}

#[get("/setup")]
pub async fn setup(site_data: SiteData) -> Result<Template, ErrorPage> {
    let ctx = context! { site_data };
    Ok(Template::render("site/setup", ctx))
}

#[derive(FromForm)]
pub struct SetupForm {
    pub username: String,
    pub password: String,
    pub password_verify: String,
    pub show_nsfw: bool,
    pub email: Option<String>,
    pub site_name: String,
    pub site_description: Option<String>,
}

#[post("/do_setup", data = "<form>")]
pub async fn do_setup(
    form: Form<SetupForm>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    let register_form = RegisterForm {
        username: form.username.clone(),
        password: form.password.clone(),
        password_verify: form.password_verify.clone(),
        show_nsfw: form.show_nsfw,
        ..Default::default()
    };
    let jwt = register(register_form).await?.jwt.unwrap().into_inner();
    cookies.add(Cookie::new("jwt", jwt.clone()));

    create_site(form.site_name.clone(), form.site_description.clone(), jwt).await?;

    Ok(Redirect::to(uri!("/")))
}

#[get("/legal")]
pub async fn legal(site_data: SiteData) -> Result<Template, ErrorPage> {
    let message = site_data
        .site
        .site_view
        .as_ref()
        .map(|s| s.site.legal_information.clone());
    let ctx = context! { message, site_data };
    Ok(Template::render("message", ctx))
}

#[get("/search?<keywords>")]
pub async fn search(keywords: String, site_data: SiteData) -> Result<Template, ErrorPage> {
    let search_results = crate::api::site::search(keywords.clone(), site_data.auth.clone()).await?;
    let search_results_count = search_results.users.len()
        + search_results.communities.len()
        + search_results.posts.len()
        + search_results.comments.len();
    let ctx = context! { site_data, keywords, search_results, search_results_count };
    Ok(Template::render("site/search", ctx))
}
