use rocket::{response::Responder, Request};
use rocket_dyn_templates::{context, Template};

#[derive(Debug)]
pub struct ErrorPage(anyhow::Error);

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ErrorPage {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        warn!("{}", self.0);
        let ctx = context! { error: self.0.to_string() };
        Template::render("error", ctx).respond_to(request)
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
