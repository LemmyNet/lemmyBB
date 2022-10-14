use crate::{env::external_domain, pagination::PAGE_ITEMS};
use chrono::NaiveDateTime;
use comrak::ComrakOptions;
use lemmy_db_schema::{
    newtypes::CommentId,
    source::{community::CommunitySafe, person::PersonSafe},
};
use lemmy_db_views::structs::CommentView;
use once_cell::sync::Lazy;
use rocket_dyn_templates::handlebars::{
    handlebars_helper,
    Context,
    Handlebars,
    Helper,
    Output,
    RenderContext,
    RenderError,
};

static COMRAK: Lazy<ComrakOptions> = Lazy::new(|| {
    let mut comrak = ComrakOptions::default();
    comrak.extension.autolink = true;
    comrak
});

#[rustfmt::skip]
pub fn replace_smilies(text: &str) -> String {
    text
        .replace(":D", "![]($DOMAIN/assets/images/smilies/icon_e_biggrin.gif)")
        .replace(":)", "![]($DOMAIN/assets/images/smilies/icon_e_smile.gif)")
        .replace(";)", "![]($DOMAIN/assets/images/smilies/icon_e_wink.gif)")
        .replace(":(", "![]($DOMAIN/assets/images/smilies/icon_e_sad.gif)")
        .replace(":oops:", "![]($DOMAIN/assets/images/smilies/icon_redface.gif)")
        .replace(":o", "![]($DOMAIN/assets/images/smilies/icon_e_surprised.gif)")
        .replace(":shock:", "![]($DOMAIN/assets/images/smilies/icon_eek.gif)")
        .replace(":?", "![]($DOMAIN/assets/images/smilies/icon_e_confused.gif)")
        .replace("8-)", "![]($DOMAIN/assets/images/smilies/icon_cool.gif)")
        .replace(":lol:", "![]($DOMAIN/assets/images/smilies/icon_lol.gif)")
        .replace(":x", "![]($DOMAIN/assets/images/smilies/icon_mad.gif)")
        .replace(":P", "![]($DOMAIN/assets/images/smilies/icon_razz.gif)")
        .replace(":cry:", "![]($DOMAIN/assets/images/smilies/icon_cry.gif)")
        .replace(":evil:", "![]($DOMAIN/assets/images/smilies/icon_evil.gif)")
        .replace(":twisted:", "![]($DOMAIN/assets/images/smilies/icon_twisted.gif)")
        .replace(":roll:", "![]($DOMAIN/assets/images/smilies/icon_rolleyes.gif)")
        .replace(":!:", "![]($DOMAIN/assets/images/smilies/icon_exclaim.gif)")
        .replace(":?:", "![]($DOMAIN/assets/images/smilies/icon_question.gif)")
        .replace(":idea:", "![]($DOMAIN/assets/images/smilies/icon_idea.gif)")
        .replace(":arrow:", "![]($DOMAIN/assets/images/smilies/icon_arrow.gif)")
        .replace(":|", "![]($DOMAIN/assets/images/smilies/icon_neutral.gif)")
        .replace(":mrgreen:", "![]($DOMAIN/assets/images/smilies/icon_mrgreen.gif)")
        .replace(":geek:", "![]($DOMAIN/assets/images/smilies/icon_e_geek.gif)")
        .replace(":ugeek:", "![]($DOMAIN/assets/images/smilies/icon_e_ugeek.gif)")
        .replace("$DOMAIN", &external_domain())
}

handlebars_helper!(timestamp_machine: |ts: NaiveDateTime| {
    ts.format("%Y-%m-%dT%H:%M:%S%.f+00:00").to_string()
});

handlebars_helper!(timestamp_human: |ts: NaiveDateTime| {
    // Wed Oct 05, 2022 9:17 pm
    ts.format("%a %v %R").to_string()
});

handlebars_helper!(eq: |a: i32, b: i32| {
    a == b
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
    let index = comments.iter().position(|c| c.comment.id == comment_id).unwrap();
    (index as f32 / PAGE_ITEMS as f32).floor()
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

handlebars_helper!(user_actor_id: |p: PersonSafe| {
    if p.local {
        format!("@{}", p.name)
    } else {
        format!("@{}@{}", p.name, p.actor_id.domain().unwrap())
    }
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
