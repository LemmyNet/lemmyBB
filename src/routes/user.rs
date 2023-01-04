use crate::{
    api,
    api::{
        image::upload_image,
        user::{change_password, get_captcha, get_person, mark_all_as_read, save_settings},
        NameOrId,
    },
    routes::{auth, build_jwt_cookie, ErrorPage},
    site_fairing::SiteData,
    template_helpers::i18n_,
    utils::{empty_to_opt, main_site_title, Context},
    ALL_LANGUAGES,
};
use lemmy_api_common::{
    person::{ChangePassword, SaveUserSettings},
    sensitive::Sensitive,
};
use rocket::{
    form::Form,
    fs::TempFile,
    http::{Cookie, CookieJar},
    response::Redirect,
    Either,
};
use rocket_dyn_templates::{context, Template};

#[get("/login")]
pub async fn login(site_data: SiteData) -> Result<Template, ErrorPage> {
    let ctx = Context::builder()
        .title(format!("Login - {}", site_data.site.site_view.site.name))
        .site_data(site_data)
        .other(())
        .build();
    Ok(Template::render("user/login", ctx))
}

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[post("/login", data = "<form>")]
pub async fn do_login(
    form: Form<LoginForm>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    let jwt = api::user::login(&form.username, &form.password)
        .await?
        .jwt
        .unwrap();
    cookies.add(build_jwt_cookie(jwt));
    Ok(Redirect::to(uri!("/")))
}

#[get("/register")]
pub async fn register(site_data: SiteData) -> Result<Template, ErrorPage> {
    let captcha = get_captcha().await?;
    let ctx = Context::builder()
        .title(format!("Register - {}", site_data.site.site_view.site.name))
        .site_data(site_data)
        .other(context! { captcha })
        .build();
    Ok(Template::render("user/register", ctx))
}

#[derive(FromForm, Default)]
pub struct RegisterForm {
    pub username: String,
    pub password: String,
    pub password_verify: String,
    pub show_nsfw: bool,
    pub email: Option<String>,
    pub captcha_uuid: Option<String>,
    pub captcha_answer: Option<String>,
    pub honeypot: Option<String>,
    pub application_answer: Option<String>,
    pub refresh_captcha: Option<String>,
}

#[post("/register", data = "<form>")]
pub async fn do_register(
    mut form: Form<RegisterForm>,
    cookies: &CookieJar<'_>,
    site_data: SiteData,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    if form.refresh_captcha.is_some() {
        // user requested new captcha, so reload page
        return Ok(Either::Right(Redirect::to(uri!(register))));
    }

    // empty fields gets parsed into Some(""), convert that to None
    form.captcha_answer = form.captcha_answer.clone().filter(|h| !h.is_empty());
    form.honeypot = form.honeypot.clone().filter(|h| !h.is_empty());
    form.email = form.email.clone().filter(|h| !h.is_empty());
    form.application_answer = form.application_answer.clone().filter(|h| !h.is_empty());

    let res = api::user::register(form.into_inner()).await?;
    let message = if let Some(jwt) = res.jwt {
        cookies.add(build_jwt_cookie(jwt));
        return Ok(Either::Right(Redirect::to(uri!("/"))));
    } else if res.verify_email_sent {
        i18n_(&site_data, "registration_confirm_email")
    } else {
        i18n_(&site_data, "registration_admin_approval")
    };

    let ctx = Context::builder()
        .title(main_site_title(&site_data.site))
        .site_data(site_data)
        .other(context! { message })
        .build();
    Ok(Either::Left(Template::render("message", ctx)))
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    // simply delete the cookie
    cookies.remove(Cookie::named("jwt"));
    Ok(Redirect::to(uri!("/")))
}

#[post("/mark_all_notifications_read")]
pub async fn mark_all_notifications_read(cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    mark_all_as_read(auth(cookies).unwrap()).await?;
    Ok(Redirect::to(uri!("/")))
}

#[get("/view_profile?<u>")]
pub async fn view_profile(u: i32, site_data: SiteData) -> Result<Template, ErrorPage> {
    let person = get_person(NameOrId::Id(u), site_data.auth.clone()).await?;
    let display_name = person.person_view.person.display_name.clone();
    let name = person.person_view.person.name.clone();
    let ctx = Context::builder()
        .title(format!(
            "Viewing profile - {}",
            display_name.unwrap_or(name)
        ))
        .site_data(site_data)
        .other(context! { person })
        .build();
    Ok(Template::render("user/view_profile", ctx))
}

#[derive(FromForm, Debug)]
pub struct EditProfileForm<'r> {
    pub displayname: String,
    pub language: String,
    // the signature
    pub message: String,
    pub avatar_delete: bool,
    pub avatar_upload_file: TempFile<'r>,
    pub email: String,
    pub new_password: String,
    pub confirm_password: String,
    pub cur_password: String,
}

#[get("/edit_profile")]
pub async fn edit_profile(site_data: SiteData) -> Result<Template, ErrorPage> {
    let mut all_languages = ALL_LANGUAGES.to_vec();
    let l = i18n_(&site_data, "browser_default_language");
    all_languages.push(("browser", &l));
    let ctx = Context::builder()
        .title("Edit profile")
        .site_data(site_data)
        .other(context! { all_languages })
        .build();
    Ok(Template::render("user/edit_profile", ctx))
}

#[post("/edit_profile", data = "<form>")]
pub async fn do_edit_profile(
    mut form: Form<EditProfileForm<'_>>,
    site_data: SiteData,
) -> Result<Template, ErrorPage> {
    let auth = site_data.auth.clone().unwrap();
    let mut params = SaveUserSettings {
        display_name: empty_to_opt(form.displayname.clone()),
        email: empty_to_opt(form.email.clone()).map(Sensitive::new),
        bio: empty_to_opt(form.message.clone()),
        interface_language: empty_to_opt(form.language.clone()),
        auth: auth.clone(),
        ..Default::default()
    };
    if form.avatar_delete {
        params.avatar = Some("".to_string());
    }
    if form.avatar_upload_file.len() != 0 {
        let avatar = upload_image(&mut form.avatar_upload_file, auth.clone(), &site_data).await?;
        params.avatar = Some(avatar.to_string());
    }
    save_settings(params).await?;

    if !form.new_password.is_empty()
        && !form.confirm_password.is_empty()
        && !form.cur_password.is_empty()
    {
        let params = ChangePassword {
            new_password: Sensitive::new(form.new_password.clone()),
            new_password_verify: Sensitive::new(form.confirm_password.clone()),
            old_password: Sensitive::new(form.cur_password.clone()),
            auth,
        };
        change_password(params).await?;
    }
    let message = i18n_(&site_data, "settings_updated");
    let ctx = Context::builder()
        .title(message.clone())
        .site_data(site_data)
        .other(context! { message })
        .build();
    Ok(Template::render("message", ctx))
}
