use json_gettext::{JSONGetText, JSONGetTextBuilder};
use once_cell::sync::OnceCell;
use rocket::{http::Status, response::Responder, Request};
use rocket_dyn_templates::{context, Template};

#[derive(Debug)]
pub struct ErrorPage(anyhow::Error);

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ErrorPage {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        warn!("{}", self.0);
        let error = localize_error_message(self.0.to_string());
        let template = Template::render("error", context! { error });
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

// TODO: need to pass in SiteData to get actual user language
fn localize_error_message(key: String) -> String {
    static LANG_CELL: OnceCell<JSONGetText> = OnceCell::new();
    let langs = LANG_CELL.get_or_init(|| {
        let mut builder = JSONGetTextBuilder::new("en");
        // TODO: when adding other languages, getting many errors TextInKeyNotInDefaultKey
        //       because translation key is included in eg ru.json, but not en.json
        builder
            .add_json_file("en", "lemmy-translations/translations/en.json")
            .unwrap();
        builder.build().unwrap()
    });
    get_text!(langs, "en", &key)
        .map(|t| t.to_string())
        .unwrap_or(key)
}
