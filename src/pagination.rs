use serde::Serialize;

#[derive(Serialize)]
pub struct Pagination {
    pub current_page: i64,
    pub is_last_page: bool,
    pub base_link: &'static str,
    before_pages: Vec<i64>,
    after_pages: Vec<i64>,
    previous_separator: bool,
}

impl Pagination {
    pub fn new(current_page: i64, is_last_page: bool, base_link: &'static str) -> Pagination {
        let before_pages: Vec<i64> = vec![current_page - 2, current_page - 1]
            .into_iter()
            .filter(|p| p > &1)
            .collect();
        let mut after_pages = if !is_last_page {
            vec![current_page + 1, current_page + 2]
        } else {
            vec![]
        };
        if before_pages.len() <= 1 {
            after_pages.push(current_page + 3);
        }
        if before_pages.is_empty() {
            after_pages.push(current_page + 4);
        }
        Pagination {
            current_page,
            is_last_page,
            base_link,
            before_pages,
            after_pages,
            previous_separator: current_page >= 5,
        }
    }
}
