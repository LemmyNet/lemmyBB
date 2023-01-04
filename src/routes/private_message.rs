use crate::{
    api::{
        private_message::{
            create_private_message,
            list_private_messages,
            mark_private_message_read,
        },
        user::get_person,
        NameOrId,
    },
    error::ErrorPage,
    site_fairing::SiteData,
    utils::{replace_smilies, Context},
};
use chrono::NaiveDateTime;
use futures::future::join_all;
use itertools::Itertools;
use lemmy_api_common::{
    lemmy_db_schema::{newtypes::PersonId, source::person::PersonSafe},
    lemmy_db_views::structs::PrivateMessageView,
    private_message::PrivateMessageResponse,
};
use rocket::{form::Form, response::Redirect, Either};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use std::hash::{Hash, Hasher};

#[derive(Serialize)]
struct PrivateMessageThread {
    pub other_participant: PersonSafe,
    pub read: bool,
    pub last_message: NaiveDateTime,
}

// TODO: add these derives directly in lemmy and remove wrapper
#[derive(PartialEq, Debug)]
struct PersonSafeWrapper(PersonSafe);

impl Eq for PersonSafeWrapper {}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for PersonSafeWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.actor_id.hash(state)
    }
}

#[get("/private_messages")]
pub async fn private_messages_list(site_data: SiteData) -> Result<Template, ErrorPage> {
    let my_user_id = site_data
        .site
        .my_user
        .as_ref()
        .unwrap()
        .local_user_view
        .person
        .id;
    let auth = site_data.auth.clone().unwrap();
    let private_message_threads: Vec<_> = list_private_messages(false, auth)
        .await?
        .private_messages
        .into_iter()
        // group by the other user involved in this pm
        .into_group_map_by(|pm| {
            if my_user_id != pm.creator.id {
                PersonSafeWrapper(pm.creator.clone())
            } else {
                // HACK: convert from PersonSafeAlias1 to PersonSafe
                PersonSafeWrapper(
                    serde_json::from_str(&serde_json::to_string(&pm.recipient.clone()).unwrap())
                        .unwrap(),
                )
            }
        })
        .into_iter()
        .map(|(user, pms)| {
            PrivateMessageThread {
                other_participant: user.0,
                // messages sent by own user are always unread
                // https://github.com/LemmyNet/lemmy/issues/2484
                read: pms
                    .iter()
                    .filter(|pm| pm.creator.id != my_user_id)
                    .all(|pm| pm.private_message.read),
                last_message: pms
                    .iter()
                    .map(|pm| pm.private_message.published)
                    .max()
                    .unwrap(),
            }
        })
        // newest messages first
        .sorted_by_key(|pmt| -pmt.last_message.timestamp())
        .collect();
    let ctx = Context::builder()
        .title("View messages")
        .site_data(site_data)
        .other(context!(private_message_threads))
        .build();
    Ok(Template::render("private_message/overview", ctx))
}

#[get("/private_messages_thread?<u>")]
pub async fn private_messages_thread(u: i32, site_data: SiteData) -> Result<Template, ErrorPage> {
    let other_user_id = PersonId(u);
    let auth = site_data.auth.clone().unwrap();
    // TODO: would be nice if lemmy api could query PMs involving given user
    let private_messages: Vec<PrivateMessageView> = list_private_messages(false, auth.clone())
        .await?
        .private_messages
        .into_iter()
        .filter(|pm| pm.creator.id == other_user_id || pm.recipient.id == other_user_id)
        .sorted_by_key(|pm| pm.private_message.published)
        .collect();

    // mark as read
    let my_user_id = site_data
        .site
        .my_user
        .as_ref()
        .unwrap()
        .local_user_view
        .person
        .id;
    join_all(
        private_messages
            .iter()
            .filter(|pm| !pm.private_message.read)
            // messages sent by own user are also errouneously marked as unread
            // https://github.com/LemmyNet/lemmy/issues/2484
            .filter(|pm| pm.creator.id != my_user_id)
            .map(|pm| mark_private_message_read(pm.private_message.id, auth.clone())),
    )
    .await
    .into_iter()
    .collect::<Result<Vec<PrivateMessageResponse>, anyhow::Error>>()?;

    let ctx = Context::builder()
        .title("Private messages thread")
        .site_data(site_data)
        .other(context!(private_messages, other_user_id))
        .build();
    Ok(Template::render("private_message/thread", ctx))
}

#[get("/private_messages_editor?<u>")]
pub async fn private_message_editor(u: i32, site_data: SiteData) -> Result<Template, ErrorPage> {
    render_editor(u, None, site_data).await
}

pub async fn render_editor(
    u: i32,
    message: Option<String>,
    site_data: SiteData,
) -> Result<Template, ErrorPage> {
    let recipient = get_person(NameOrId::Id(u), site_data.auth.clone())
        .await?
        .person_view
        .person;
    let ctx = Context::builder()
        .title("Compose private message")
        .site_data(site_data)
        .other(context!(recipient, message))
        .build();
    Ok(Template::render("private_message/editor", ctx))
}

#[derive(FromForm)]
pub struct PrivateMessageForm {
    message: String,
    preview: Option<String>,
}

#[post("/send_private_message?<u>", data = "<form>")]
pub async fn do_send_private_message(
    u: i32,
    form: Form<PrivateMessageForm>,
    site_data: SiteData,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    let message = replace_smilies(&form.message, &site_data);

    if form.preview.is_some() {
        return Ok(Either::Left(
            render_editor(u, Some(message), site_data).await?,
        ));
    }

    create_private_message(message, PersonId(u), site_data.auth.unwrap()).await?;
    Ok(Either::Right(Redirect::to(uri!(private_messages_thread(
        u
    )))))
}
