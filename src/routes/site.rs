use crate::{
    api::{
        community::list_communities,
        extra::{get_last_reply_in_community, PostOrComment},
        site::{create_site, get_site},
        user::register,
    },
    routes::{auth, user::RegisterForm, ErrorPage},
};
use anyhow::Error;
use futures::future::join_all;
use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    response::Redirect,
    Either,
};
use rocket_dyn_templates::{context, Template};

#[get("/")]
pub async fn index(cookies: &CookieJar<'_>) -> Result<Either<Redirect, Template>, ErrorPage> {
    let site = get_site(cookies).await?;
    if site.0.site_view.is_none() {
        // need to setup site
        return Ok(Either::Left(Redirect::to(uri!(setup))));
    }

    let mut communities = list_communities(auth(cookies)).await?;
    communities
        .communities
        .sort_unstable_by_key(|c| c.community.id.0);
    let last_replies = join_all(
        communities
            .communities
            .iter()
            .map(|c| get_last_reply_in_community(c.community.id, auth(cookies))),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<Option<PostOrComment>>, Error>>()?;

    let ctx = context! { site, communities, last_replies };
    Ok(Either::Right(Template::render("index", ctx)))
}

#[get("/setup")]
pub async fn setup(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = get_site(cookies).await?;
    let ctx = context! { site };
    Ok(Template::render("setup", ctx))
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

    Ok(Redirect::to(uri!(index)))
}

#[get("/legal")]
pub async fn legal(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site = get_site(cookies).await?;
    let message = site
        .0
        .site_view
        .as_ref()
        .map(|s| s.site.legal_information.clone());
    let ctx = context! { message, site };
    Ok(Template::render("message", ctx))
}
