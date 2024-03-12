pub mod page;
pub mod pagecrawler;

use url::Url;

// url must have a domain!
pub fn is_in_domain(url: &Url) -> bool {
    url.domain().unwrap().ends_with(".neocities.org")
}

// url and siteurl must have a domain!
pub fn is_in_site(url: &Url, siteurl: &Url) -> bool {
    url.domain() == siteurl.domain()
}
