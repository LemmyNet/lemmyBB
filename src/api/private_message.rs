use crate::api::{get, post};
use anyhow::Error;
use lemmy_api_common::{
    person::{
        GetPrivateMessages,
        MarkPrivateMessageAsRead,
        PrivateMessageResponse,
        PrivateMessagesResponse,
    },
    sensitive::Sensitive,
};
use lemmy_db_schema::newtypes::PrivateMessageId;

pub(crate) async fn list_private_messages(
    unread_only: bool,
    auth: Sensitive<String>,
) -> Result<PrivateMessagesResponse, Error> {
    let params = GetPrivateMessages {
        auth,
        unread_only: Some(unread_only),
        ..Default::default()
    };
    get("/private_message/list", params).await
}

pub(crate) async fn mark_private_message_read(
    private_message_id: PrivateMessageId,
    auth: Sensitive<String>,
) -> Result<PrivateMessageResponse, Error> {
    let params = MarkPrivateMessageAsRead {
        private_message_id,
        read: true,
        auth,
    };
    post("/private_message/mark_as_read", params).await
}
