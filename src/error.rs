use crate::{site_fairing::SiteData, utils::Context};
use json_gettext::{JSONGetText, JSONGetTextBuilder};
use once_cell::sync::OnceCell;
use rocket::{http::Status, response::Responder, Request};
use rocket_dyn_templates::{context, Template};
use std::fs::read_dir;

#[derive(Debug)]
pub struct ErrorPage(anyhow::Error);

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ErrorPage {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        warn!("{}", self.0);
        let site: &Option<SiteData> = request.local_cache(|| None::<SiteData>);
        let error = self.0.to_string();
        let template = match site {
            Some(site_data) => {
                let message = localize_error_message(error, &site_data.lang);
                let ctx = Context::builder()
                    .title(message)
                    .site_data(site_data.clone())
                    .other(())
                    .build();
                Template::render("message", ctx)
            }
            None => {
                let title = error.clone();
                Template::render("error", context! { title, error })
            }
        };
        let mut res = template.respond_to(request)?;
        res.set_status(Status::InternalServerError);
        Ok(res)
    }
}

impl<T> From<T> for ErrorPage
where
    T: Into<anyhow::Error>,
{
    fn from(t: T) -> Self {
        ErrorPage(t.into())
    }
}

fn localize_error_message(key: String, lang: &str) -> String {
    static LANG_CELL: OnceCell<JSONGetText> = OnceCell::new();
    let langs_list = LANG_CELL.get_or_init(|| {
        let mut builder = JSONGetTextBuilder::new("en");
        for file in read_dir("lemmy-translations/translations/").unwrap() {
            let file = file.unwrap();
            let key = file.file_name().to_str().unwrap().replace(".json", "");
            // Workaround for https://github.com/magiclen/json-gettext/issues/1
            let ignored = [
                "lt", "ar", "uk", "ru", "cy", "sr_Latn", "sk", "pl", "cs", "ga",
            ];
            if !ignored.contains(&&*key) {
                builder.add_json_file(key, file.path()).ok();
            }
        }
        builder.build().unwrap()
    });
    get_text!(langs_list, lang, &key)
        .map(|t| t.to_string())
        .unwrap_or(key)
}

#[test]
fn localize_error() {
    assert_eq!(
        "No se pudo encontrar ese nombre de usuario o correo electrónico.",
        localize_error_message("couldnt_find_that_username_or_email".to_string(), "es")
    );
    assert_eq!(
        "missing_key",
        localize_error_message("missing_key".to_string(), "fr")
    );
}
