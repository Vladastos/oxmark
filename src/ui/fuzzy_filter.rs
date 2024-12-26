use nucleo_matcher::{pattern::Pattern, Config, Matcher};
use nucleo_matcher::pattern::Normalization;
use nucleo_matcher::pattern::CaseMatching;
use super::Bookmark;

pub fn get_filtered_bookmarks(bookmarks: Vec<Bookmark>, search_term: String) -> Vec<Bookmark> {
    let mut matcher =Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(&search_term, CaseMatching::Ignore, Normalization::Smart);
    
    let mut filtered_bookmarks: Vec<Bookmark> = Vec::new();
    
    for bookmark in bookmarks {
        let matches = pattern.match_list(vec![bookmark.name.clone().unwrap_or("".to_string())], &mut matcher);
        if matches.len() > 0 {
            filtered_bookmarks.push(bookmark);
        }    
    }

    filtered_bookmarks
    
}