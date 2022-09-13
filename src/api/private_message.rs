use crate::api::get;
use anyhow::Error;
use lemmy_api_common::{
    person::{GetPrivateMessages, PrivateMessagesResponse},
    sensitive::Sensitive,
};

pub(crate) async fn list_private_messages(
    auth: Sensitive<String>,
) -> Result<PrivateMessagesResponse, Error> {
    let params = GetPrivateMessages {
        auth,
        unread_only: Some(true),
        ..Default::default()
    };
    get("/private_message/list", params).await
}
