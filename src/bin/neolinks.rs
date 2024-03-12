use std::env;
use url::Url;
use neomap::{page::Page, pagecrawler, pagecrawler::PageCrawler};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut domain = String::new();
    let mut is_recursive = false;
    let mut is_html_only = false;

    /* find and parse any options */
    let options: Vec<&String> = args.iter().filter(|&a| a.starts_with("-")).collect();
    for o in options {
        if o.starts_with("-d") {
            domain = o.chars().skip(2).collect();
        } else if o == "-r" {
            is_recursive = true;
        } else if o == "-h" {
            is_html_only = true;
        }
    }

    /* get anything that comes after options */
    let sites: Vec<&String> = args.iter().skip_while(|&a| a.starts_with("-")).collect();

    let mut links: Vec<Url> = Vec::new();
    for s in sites {
        if is_recursive {
            links = crawl_site(s);
        } else {
            links = crawl_page(s);
        }
    }

    output(&mut links, &domain, is_html_only);
}

fn crawl_site(site: &str) -> Vec<Url> {
    let url = Url::parse(site).unwrap();
    let mut crawler = match PageCrawler::new(url) {
        Ok(c) => c,
        Err(_) => todo!(),
    };
    crawler.crawl();

    crawler.get_links()
}

fn crawl_page(site: &str) -> Vec<Url> {
    let url = Url::parse(site).unwrap();
    let mut page = Page::new(url.clone()).unwrap();
    let _ = page.fetch();
    page.get_links()
}

fn is_in_domain(url: &Url, domain: &str) -> bool {
    match url.domain() {
        Some(d) => d.ends_with(domain),
        None => false,
    }
}

fn output(links: &mut Vec<Url>, domain: &str, is_html_only: bool) {
    links.sort();
    links.dedup();
    for l in links {
        if !is_in_domain(&l, domain) {
            continue;
        }

        if is_html_only && !pagecrawler::is_url_html(l) {
            continue;
        }

        println!("{}", l.as_str());
    }
}
