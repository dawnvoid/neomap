mod database;
use chrono::Utc;
use database::{Database, LinkEntry, SiteEntry};
use std::path::Path;
use url::Url;

fn add_site(db: &Database, url: &str, crawltime: i64) {
    let url = Url::parse(url).unwrap();
    let s = SiteEntry::new(url, crawltime).unwrap();
    db.set_site(s).unwrap();
}

fn add_link(db: &Database, srcurl: &str, dsturl: &str) {
    let src = Url::parse(srcurl).unwrap();
    let dst = Url::parse(dsturl).unwrap();
    let l = LinkEntry::new(src, dst).unwrap();
    db.set_link(l).unwrap();
}

fn main() {
    println!("Hello, world!");

    let dbpath = Path::new("neomap.db");
    let db = Database::connect(dbpath).unwrap();

    add_site(&db, "https://dawnvoid.neocities.org/", 100);
    add_site(&db, "https://errormine.neocities.org/", 100);
    add_site(&db, "https://koyo.neocities.org/", 100);
    add_site(&db, "https://undoified.neocities.org/", 100);
    add_site(&db, "https://kryptonaut.neocities.org/", 100);

    add_link(
        &db,
        "https://dawnvoid.neocities.org/",
        "https://errormine.neocities.org/",
    );
    add_link(
        &db,
        "https://dawnvoid.neocities.org/",
        "https://koyo.neocities.org/",
    );
    add_link(
        &db,
        "https://undoified.neocities.org/",
        "https://kryptonaut.neocities.org/",
    );

    // db.delete_link_by_srcurl(
    //     LinkEntry::new(
    //         Url::parse("https://dawnvoid.neocities.org/").unwrap(),
    //         Url::parse("https://example.org/").unwrap(),
    //     )
    //     .unwrap(),
    // )
    // .unwrap();

    db.delete_site_by_url(
        SiteEntry::new(
            Url::parse("https://dawnvoid.neocities.org/").unwrap(),
            0,
        )
        .unwrap(),
    )
    .unwrap();

    let r = db.get_site_with_oldest_crawltime().unwrap().unwrap();
    println!("site with oldest crawltime: {r:?}");

    let s = SiteEntry::new(
        Url::parse("https://errormine.neocities.org/").unwrap(),
        Utc::now().timestamp(),
    )
    .unwrap();
    db.update_site_crawltime(s).unwrap();

    let r = db.get_site_with_oldest_crawltime().unwrap().unwrap();
    println!("site with oldest crawltime: {r:?}");

    db.disconnect().unwrap();
}
