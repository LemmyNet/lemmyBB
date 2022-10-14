use serde::Serialize;

pub static PAGE_ITEMS: i32 = 20;

/// need to represent things in a more complicated way, becayse handlebars doesnt support enums
#[derive(Serialize, Debug, Default)]
pub struct Pagination {
    pub current_page: i32,
    pub base_link: String,
    has_known_limit: bool,
    is_last_page: bool,
    last_page: i32,
    before_pages: Vec<i32>,
    after_pages: Vec<i32>,
    before_separator: bool,
    after_separator: bool,
}

#[derive(Serialize, Debug)]
pub enum PageLimit {
    // param is index of last page
    Known(i32),
    // param is the number of items on current page
    Unknown(usize),
}

impl Pagination {
    pub fn new<S: Into<String>>(
        current_page: i32,
        total_pages: PageLimit,
        base_link: S,
    ) -> Pagination {
        let mut p = Pagination {
            current_page,
            base_link: base_link.into(),
            has_known_limit: matches!(total_pages, PageLimit::Known(_)),
            before_separator: current_page >= 5,
            ..Default::default()
        };
        p.before_pages = vec![current_page - 2, current_page - 1]
            .into_iter()
            .filter(|p| p > &1)
            .collect();

        match total_pages {
            PageLimit::Known(last_page) => {
                p.after_pages = vec![current_page + 1, current_page + 2]
                    .into_iter()
                    .filter(|p| p < &last_page)
                    .collect();
                p.is_last_page = current_page == last_page;
                p.after_separator = (last_page - current_page) > 3;
                p.last_page = last_page;
            }
            PageLimit::Unknown(current_page_items) => {
                p.is_last_page = (current_page_items as i32) < PAGE_ITEMS;
                if !p.is_last_page {
                    p.after_pages.push(current_page + 1);
                    p.after_pages.push(current_page + 2);
                    if p.before_pages.len() <= 1 {
                        p.after_pages.push(current_page + 3);
                    }
                    if p.before_pages.is_empty() {
                        p.after_pages.push(current_page + 4);
                    }
                };
                p.after_separator = !p.is_last_page;
                p.last_page = -1;
            }
        };
        p
    }
}
