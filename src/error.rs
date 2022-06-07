use rocket::{http::Status, response::Responder, Request};

#[derive(Debug)]
pub struct ErrorPage(anyhow::Error);

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ErrorPage {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        warn!("{}", self.0);
        Err(Status::InternalServerError)
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
