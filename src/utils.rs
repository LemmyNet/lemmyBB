use crate::site_fairing::SiteData;
use lemmy_api_common::site::GetSiteResponse;
use serde::Serialize;
use typed_builder::TypedBuilder;

pub fn base_url(site_data: &SiteData) -> String {
    let site_actor = &site_data.site.site_view.site.actor_id;
    let origin = site_actor.scheme();
    let domain = site_actor.domain().unwrap();
    format!("{origin}://{domain}")
}

#[rustfmt::skip]
pub fn replace_smilies(text: &str, site_data: &SiteData) -> String {
    let base_url = base_url(site_data);
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
        .replace("$DOMAIN", &base_url)
}

// https://github.com/SergioBenitez/Rocket/issues/2372
pub fn empty_to_opt(value: String) -> Option<String> {
    if value.trim() == "" {
        None
    } else {
        Some(value)
    }
}

#[derive(TypedBuilder, Serialize)]
pub struct Context<T: Into<String>, R: Serialize> {
    title: T,
    site_data: SiteData,
    #[serde(flatten)]
    other: R,
}

pub fn main_site_title(site: &GetSiteResponse) -> String {
    let site = &site.site_view.site;
    if let Some(description) = &site.description {
        format!("{} - {}", site.name, description)
    } else {
        site.name.clone()
    }
}
