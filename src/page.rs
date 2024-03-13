use regex::Regex;
use reqwest::blocking;
use url::{ParseError, Url};

pub struct Page {
    pub url: Url,
    pub html: String,
}

impl Page {
    pub fn new(url: Url) -> Option<Page> {
        assert!(!url.cannot_be_a_base());
        Some(Page {
            url: url,
            html: String::new(),
        })
    }

    pub fn fetch(&mut self) -> &str {
        let response = match blocking::get(self.url.clone()) {
            Ok(r) => r,
            Err(e) => panic!("http get failed: {}", e.to_string()),
        };
        let html = match response.text() {
            Ok(t) => t,
            Err(e) => panic!("text extraction failed: {}", e.to_string()),
        };
        self.html = html;
        &self.html
    }

    pub fn get_links(&self) -> Vec<Url> {
        let mut links = get_href_links(&self.html);
        links.append(&mut get_src_links(&self.html));
        links.sort_unstable();
        links.dedup();

        let mut urls: Vec<Url> = Vec::with_capacity(links.len());
        for l in links {
            let u = match Url::parse(l) {
                Ok(u) => u,
                Err(ParseError::RelativeUrlWithoutBase) => match self.url.join(l) {
                    Ok(u) => u,
                    Err(e) => panic!("{e:?}"),
                },
                Err(e) => {
                    println!(r#"failed to parse url "{l}": {e:?}"#);
                    continue;
                }
            };

            /* if not base, assume it's relative and join with page url as base */
            let mut based = u;
            if based.cannot_be_a_base() {
                based = match self.url.join(based.path()) {
                    Ok(j) => j,
                    Err(_) => {
                        println!(r#"failed to join non-base url "{l}""#);
                        continue;
                    }
                };
            }

            urls.push(based);
        }
        urls
    }
}

fn get_href_links(html: &str) -> Vec<&str> {
    // let re = Regex::new(r#"<a(\s+|\s+.*?\s+)href="(.*?)"(\s*|\s+.*?\s+)>(.*?)<\/a(\s*|\s+.*?)>"#).unwrap();
    // let re = Regex::new(r"\/[\w.-]+\/").unwrap();
    let re = Regex::new(r#"href="(?<url>.*?)""#).unwrap();
    re.captures_iter(html)
        .map(|m| m.name("url").unwrap().as_str())
        .collect()
}

fn get_src_links(html: &str) -> Vec<&str> {
    let re = Regex::new(r#"src="(?<url>.*?)""#).unwrap();
    re.captures_iter(html)
        .map(|m| m.name("url").unwrap().as_str())
        .collect()
}
