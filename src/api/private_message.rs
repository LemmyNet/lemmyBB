use crate::api::{get, post};
use anyhow::Error;
use lemmy_api_common::{
    lemmy_db_schema::newtypes::{PersonId, PrivateMessageId},
    private_message::{
        CreatePrivateMessage,
        GetPrivateMessages,
        MarkPrivateMessageAsRead,
        PrivateMessageResponse,
        PrivateMessagesResponse,
    },
    sensitive::Sensitive,
};

pub(crate) async fn list_private_messages(
    unread_only: bool,
    auth: Sensitive<String>,
) -> Result<PrivateMessagesResponse, Error> {
    let params = GetPrivateMessages {
        auth,
        unread_only: Some(unread_only),
        ..Default::default()
    };
    get("/private_message/list", &params).await
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
    post("/private_message/mark_as_read", &params).await
}

pub(crate) async fn create_private_message(
    content: String,
    recipient_id: PersonId,
    auth: Sensitive<String>,
) -> Result<PrivateMessageResponse, Error> {
    let params = CreatePrivateMessage {
        content,
        recipient_id,
        auth,
    };
    post("/private_message", &params).await
}
