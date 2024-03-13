use crate::neocrawler;
use std::collections::HashMap;
use url::Url;

pub struct NeoCrawler {
    sites: HashMap<Url, Vec<Url>>,
}

impl NeoCrawler {
    pub fn new() -> NeoCrawler {
        NeoCrawler {
            sites: HashMap::new(),
        }
    }

    pub fn crawl(&mut self, rootsite: &Url) {
        let mut frontier: Vec<Url> = Vec::new();
        frontier.push(rootsite.clone());
    }
}
