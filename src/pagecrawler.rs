use url::Url;
use crate::page::Page;

pub struct PageCrawler {
    url: Url,
    links: Vec<Url>,
    pages: Vec<Url>,
}

impl PageCrawler {
    pub fn new(url: Url) -> Result<PageCrawler, String> {
        if url.cannot_be_a_base() {
            return Err(String::from("invalid url"));
        }
        Ok(PageCrawler {url: url, links: Vec::new(), pages: Vec::new()})
    }

    pub fn crawl(&mut self) {
        let mut frontier: Vec<Url> = Vec::new();
        frontier.push(self.url.clone());

        /* perform bfs */
        while !frontier.is_empty() {
            let currenturl = frontier.pop().unwrap(); /* should never fail */

            /* only process pages that we haven't processed before */
            /* slow but i don't care right now */
            if self.pages.contains(&currenturl) {
                continue;
            }

            self.links.push(currenturl.clone());

            /* try to only visit html pages */
            let cd = currenturl.domain().unwrap();
            let d = self.url.domain().unwrap();
            if cd != d {
                continue;
            }
            if !is_url_html(&currenturl) {
                continue;
            }

            println!("processing {}", currenturl.as_str());

            self.pages.push(currenturl.clone());

            let mut currentpage = Page::new(currenturl).unwrap(); /* should never fail as long as url was constructed correctly */
            let _ = currentpage.fetch();
            frontier.append(&mut currentpage.get_links());
        }
    }

    pub fn get_links(&self) -> Vec<Url> {
        self.links.clone()
    }
}

pub fn is_url_html(url: &Url) -> bool {
    let path = url.path().to_lowercase();
    let mut ok = false;
    if path.ends_with(".html") || path.ends_with(".htm") {
        ok = true;
    }
    if !path.contains(".") { /* not perfect but good enough for now */
        ok = true;
    }
    ok
}

fn is_url_image(url: &Url) -> bool {
    let path = url.path().to_lowercase();
    if path.ends_with(".png") {
        return true;
    }
    if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        return true;
    }
    if path.ends_with(".gif") {
        return true;
    }
    if path.ends_with(".apng") {
        return true;
    }
    if path.ends_with(".tiff") || path.ends_with(".tif") {
        return true;
    }
    if path.ends_with(".jfif") {
        return true;
    }
    false
}
