use chrono::NaiveDateTime;
use lemmy_db_schema::newtypes::CommentId;
use lemmy_db_views::structs::CommentView;
use rocket_dyn_templates::handlebars::{
    handlebars_helper,
    Context,
    Handlebars,
    Helper,
    Output,
    RenderContext,
    RenderError,
};

handlebars_helper!(timestamp_machine: |ts: NaiveDateTime| {
    ts.format("%Y-%m-%dT%H:%M:%S%.f+00:00").to_string()
});

handlebars_helper!(timestamp_human: |ts: NaiveDateTime| {
    ts.format("%c").to_string()
});

handlebars_helper!(sum: |a: i32, b: i32| {
    a + b
});

handlebars_helper!(modulo: |a: i32, b: i32| {
    a % b
});

// Converts markdown to html. Use some hacks to change the generated html, so that text size
// and style are consistent with phpBB:
// - remove outer <p> wrapper
// - use <br /><br /> for newlines
// TODO: this currently breaks block quotes and maybe other things
handlebars_helper!(markdown: |md: Option<String>| {
    match md {
    Some(mut o) => {
            o = o.replace("\n\n", "\\\n");
            let mut comrak = comrak::ComrakOptions::default();
            comrak.extension.autolink = true;
            let mut x = comrak::markdown_to_html(&o, &comrak);
            x = x.replace(r"<p>", "");
            x = x.replace(r"</p>", "");
            x = x.replace("<br />", "<br /><br />");
            x
    }
        None => "".to_string()
        }
});

// Returns position of comment in thread. vec is assumed to be sorted
handlebars_helper!(comment_index: |comment_id: CommentId, comments: Vec<CommentView>| {
    comments.iter().position(|c| c.comment.id == comment_id).unwrap()
});

// https://github.com/sunng87/handlebars-rust/issues/43?ysclid=l5jxaw92um440916198#issuecomment-427482841
pub fn handlebars_helper_vec_length(
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
