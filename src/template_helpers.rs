use crate::{pagination::PAGE_ITEMS, site_fairing::SiteData};
use chrono::NaiveDateTime;
use comrak::ComrakOptions;
use json_gettext::{JSONGetText, JSONGetTextBuilder};
use lemmy_api_common::{
    lemmy_db_schema::{
        newtypes::CommentId,
        source::{community::CommunitySafe, person::PersonSafe},
    },
    lemmy_db_views::structs::CommentView,
    lemmy_db_views_actor::structs::CommunityModeratorView,
};
use once_cell::sync::{Lazy, OnceCell};
use rocket_dyn_templates::handlebars::{
    handlebars_helper,
    Context,
    Handlebars,
    Helper,
    Output,
    RenderContext,
    RenderError,
};
use serde_json::Value;

static COMRAK: Lazy<ComrakOptions> = Lazy::new(|| {
    let mut comrak = ComrakOptions::default();
    comrak.extension.autolink = true;
    comrak
});

handlebars_helper!(timestamp_machine: |ts: NaiveDateTime| {
    ts.format("%Y-%m-%dT%H:%M:%S%.f+00:00").to_string()
});

handlebars_helper!(timestamp_human: |ts: NaiveDateTime| {
    // Wed Oct 05, 2022 9:17 pm
    ts.format("%a %v %R").to_string()
});

handlebars_helper!(add: |a: i32, b: i32| {
    a + b
});

handlebars_helper!(sub: |a: i32, b: i32| {
    a - b
});

handlebars_helper!(modulo: |a: i32, b: i32| {
    a % b
});

// Returns position of comment in thread. vec is assumed to be sorted
handlebars_helper!(comment_page: |comment_id: CommentId, comments: Vec<CommentView>| {
    let index = comments.iter().position(|c| c.comment.id == comment_id);
    if let Some(i) = index {
    (i as f32 / PAGE_ITEMS as f32).ceil() as i32
        } else {
        // TODO: properly handle case of deleted parent
        -1
    }
});

// Converts markdown to html. Replace generated <p></p> with <br /><br /> for newlines, because
// otherwise fonts are rendered too big.
handlebars_helper!(markdown: |md: Option<String>| {
    match md {
    Some(m) => {
        comrak::markdown_to_html(&m, &COMRAK)
            .replace("</p>\n<p>", "<br /><br />")
            .replace(r"<p>", "")
            .replace(r"</p>", "")
    }
    None => "".to_string()
    }
});

handlebars_helper!(community_actor_id: |c: CommunitySafe| {
    if c.local {
        format!("!{}", c.name)
    } else {
        format!("!{}@{}", c.name, c.actor_id.domain().unwrap())
    }
});

handlebars_helper!(user_name: |p: PersonSafe| {
    p.display_name.unwrap_or(p.name)
});

handlebars_helper!(user_actor_id: |p: PersonSafe| {
    if p.local {
        format!("@{}", p.name)
    } else {
        format!("@{}@{}", p.name, p.actor_id.domain().unwrap())
    }
});

// Handlebars is automatically inserting spaces at line beginning in editor text area, this
// workaround avoids that.
handlebars_helper!(raw: |s: String| {
    s
});

handlebars_helper!(is_mod: |user: PersonSafe, moderators: Vec<CommunityModeratorView>| {
    moderators.iter().any(|m| m.moderator.id == user.id)
});

handlebars_helper!(is_mod_or_admin: |user: PersonSafe, moderators: Vec<CommunityModeratorView>| {
    user.admin || moderators.iter().any(|m| m.moderator.id == user.id)
});

pub fn concat(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let a = h.param(0).map(|v| v.render()).unwrap();
    let b = h.param(1).map(|v| v.value().to_string()).unwrap();

    out.write(&format!("{}{}", a, b))?;

    Ok(())
}

// https://github.com/sunng87/handlebars-rust/issues/43?ysclid=l5jxaw92um440916198#issuecomment-427482841
pub fn length(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let length = h
        .param(0)
        .as_ref()
        .and_then(|v| v.value().as_array())
        .map(|arr| arr.len())
        .ok_or_else(|| {
            RenderError::new("Param 0 with 'array' type is required for array_length helper")
        })?;

    out.write(length.to_string().as_ref())?;

    Ok(())
}

pub const ALL_LANGUAGES: [(&str, &str); 2] = [("en", "English"), ("de", "Deutsch")];

handlebars_helper!(i18n: |site_data: SiteData, key: String, *args| {
    i18n_private(site_data.lang, key, args)
});

fn i18n_private(lang: String, key: String, args: Vec<&Value>) -> String {
    static LANG_CELL: OnceCell<JSONGetText> = OnceCell::new();
    let langs = LANG_CELL.get_or_init(|| {
        let mut builder = JSONGetTextBuilder::new("en");
        for l in ALL_LANGUAGES {
            builder
                .add_json_file(
                    l.0,
                    format!("lemmybb-translations/translations/{}.json", l.0),
                )
                .unwrap();
        }
        builder.build().unwrap()
    });
    let mut text = get_text!(langs, lang, key).unwrap().to_string();
    if text.contains("{}") {
        let str = &args[2].to_string();
        text = text.replacen("{}", args[2].as_str().unwrap_or(str), 1);
    }
    text
}

pub fn i18n_(site_data: &SiteData, key: &'static str) -> String {
    i18n_private(site_data.lang.clone(), key.to_string(), vec![])
}
