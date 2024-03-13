use rusqlite::{self, Connection, OptionalExtension, Statement};
use std::path::Path;
use url::Url;

#[derive(Debug)]
pub struct Database {
    connection: Connection,
}

impl Database {
    /// Connects to a sqlite database file.
    /// If the file doesn't exist, it will be created.
    /// If the tables don't exist, they will be created.
    /// However, if the tables do exist, but have the wrong configuration,
    /// they won't be fixed.
    pub fn connect(path: &Path) -> Result<Database, String> {
        // let con = Connection::open_in_memory()?;
        let con = Connection::open(path).map_err(|e| e.to_string())?;
        let d = Database { connection: con };

        // create tables if needed
        d.try_create_tables()?;

        Ok(d)
    }

    /// Exactly the same as `Database::connect()`,
    /// but a new in-memory database is created.
    ///
    /// Intended for testing.
    pub fn connect_virtual() -> Result<Database, String> {
        let con = Connection::open_in_memory().map_err(|e| e.to_string())?;
        let d = Database { connection: con };

        // create tables if needed
        d.try_create_tables()?;

        Ok(d)
    }

    pub fn disconnect(mut self) -> Result<(), (Database, String)> {
        match self.connection.close() {
            Ok(_) => Ok(()),
            Err((c, e)) => {
                self.connection = c;
                Err((self, e.to_string()))
            }
        }
    }

    /// Attempts to create the sqlite tables.
    pub fn try_create_tables(&self) -> Result<(), String> {
        // create site table if needed
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS site (
                url TEXT NOT NULL PRIMARY KEY,
                crawltime INTEGER NOT NULL
            )",
                (),
            )
            .map_err(|e| e.to_string())?;

        // create link table if needed
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS link (
                srcurl TEXT NOT NULL,
                dsturl TEXT NOT NULL,
                PRIMARY KEY (srcurl, dsturl),
                FOREIGN KEY (srcurl) REFERENCES site (url)
                    ON UPDATE CASCADE
                    ON DELETE CASCADE
            )",
                (),
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Updates a site entry, or creates one if no site with the url exists.
    pub fn set_site(&self, site: SiteEntry) -> Result<(), String> {
        // site.insert_new(&self.connection)?;
        self.connection
            .execute(
                "INSERT INTO site (url, crawltime) VALUES (?1, ?2)
            ON CONFLICT(url) DO UPDATE SET crawltime = excluded.crawltime",
                (site.url, site.crawltime),
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Updates a link entry, or creates one if no link with the srcurl and dsturl exists.
    pub fn set_link(&self, link: LinkEntry) -> Result<(), String> {
        self.connection.execute(
            "INSERT INTO link (srcurl, dsturl) VALUES (?1, ?2)
            ON CONFLICT(srcurl, dsturl) DO UPDATE SET srcurl = excluded.srcurl, dsturl = excluded.dsturl",
            (link.srcurl, link.dsturl),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_links_by_srcurl(&self, link: LinkEntry) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM link WHERE srcurl = ?1", (link.srcurl,))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_site_by_url(&self, site: SiteEntry) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM site WHERE url = ?1", (site.url,))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_site_with_oldest_crawltime(&self) -> Result<Option<SiteEntry>, String> {
        // see https://www.db-fiddle.com/f/kUoFMMUfYyNnrpnyWWvUXG/1
        let mut statement = self
            .connection
            .prepare("SELECT url, MIN(crawltime) FROM site")
            .unwrap();
        let result = statement
            .query_row((), |row| {
                Ok(SiteEntry {
                    url: row.get(0).unwrap(),
                    crawltime: row.get(1).unwrap(),
                })
            })
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub fn update_site_crawltime(&self, site: SiteEntry) -> Result<(), String> {
        // see https://www.db-fiddle.com/f/kUoFMMUfYyNnrpnyWWvUXG/2
        let mut statement = self
            .connection
            .prepare("UPDATE site SET crawltime = ?2 WHERE url = ?1")
            .unwrap();
        let result = statement
            .execute((site.url, site.crawltime))
            .map_err(|e| e.to_string())?;
        if result != 1 {
            return Err(format!(
                "update_site_crawltime() should change exactly 1 row, but {result} were changed"
            ));
        }
        Ok(())
    }

    pub fn get_links_by_srcurl(&self, link: LinkEntry) -> Result<Vec<LinkEntry>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT srcurl, dsturl FROM link WHERE srcurl = ?1")
            .unwrap();
        let result = statement
            .query_map((link.srcurl,), |row| {
                Ok(LinkEntry {
                    srcurl: row.get(0).unwrap(),
                    dsturl: row.get(1).unwrap(),
                })
            })
            .map_err(|e| e.to_string())?;
        let resultlist: Vec<LinkEntry> = result.map(|r| r.unwrap()).collect();
        Ok(resultlist)
    }

    // pub fn get_site_by_
}

/// A site entry in a `Database`.
///
/// `url` must be properly formatted.
/// Ideally `SiteEntry::new()` should guarantee this.
///
/// `crawltime` is the unix timestamp of when the site was last crawled.
/// Sites that haven't been crawled yet should set this to 0.
#[derive(Debug, PartialEq, Eq)]
pub struct SiteEntry {
    url: String, // primary key; base url of site (e.g. "https://kryptonaut.neocities.org/")
    crawltime: i64, // timestamp of last crawl, value is irrelevant if `iscrawled` is false
}

impl SiteEntry {
    pub fn new(url: Url, lastcrawled: i64) -> Result<SiteEntry, String> {
        if url.domain().is_none() {
            return Err(format!(r#"invalid url "{}""#, url.as_str()));
        }
        let s = SiteEntry {
            url: url.to_string(),
            crawltime: lastcrawled,
        };
        Ok(s)
    }
}

/// A link entry in a `Database`.
///
/// `srcurl` and `dsturl` must be properly formatted.
/// Ideally, `LinkEntry::new()` should guarantee this.
#[derive(Debug, PartialEq, Eq)]
pub struct LinkEntry {
    srcurl: String, // source site key (this is the site that has the link)
    dsturl: String, // destination site key
}

impl LinkEntry {
    pub fn new(srcurl: Url, dsturl: Url) -> Result<LinkEntry, String> {
        if srcurl.domain().is_none() {
            return Err(format!(r#"invalid source url "{}""#, srcurl.as_ref()));
        }
        if dsturl.domain().is_none() {
            return Err(format!(r#"invalid destination url "{}""#, dsturl.as_ref()));
        }
        let l = LinkEntry {
            srcurl: String::from(srcurl.as_str()),
            dsturl: String::from(dsturl.as_str()),
        };
        Ok(l)
    }
}

#[cfg(test)]
mod tests {
    use super::{Database, LinkEntry, SiteEntry};
    use rusqlite::OptionalExtension;
    use url::Url;

    #[test]
    fn helloworld() {
        assert!(true);
    }

    fn create_site(url: &str, crawltime: i64) -> Result<SiteEntry, String> {
        let url = Url::parse(url).map_err(|e| e.to_string())?;
        let site = SiteEntry::new(url, crawltime).map_err(|e| e.to_string())?;
        Ok(site)
    }

    fn create_link(srcurl: &str, dsturl: &str) -> Result<LinkEntry, String> {
        let src = Url::parse(srcurl).map_err(|e| e.to_string())?;
        let dst = Url::parse(dsturl).map_err(|e| e.to_string())?;
        let link = LinkEntry::new(src, dst).map_err(|e| e.to_string())?;
        Ok(link)
    }

    fn get_site_by_url(db: &Database, url: &str) -> Result<Option<SiteEntry>, String> {
        let site = create_site(url, 0)?;
        let mut statement = db
            .connection
            .prepare("SELECT * FROM site WHERE url = ?1")
            .map_err(|e| e.to_string())?;
        let result = statement
            .query_row((site.url,), |row| {
                Ok(SiteEntry {
                    url: row.get(0).unwrap(),
                    crawltime: row.get(1).unwrap(),
                })
            })
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    #[test]
    fn get_site_insert() {
        let db = Database::connect_virtual().unwrap();
        let site = create_site("https://dawnvoid.neocities.org/", 0).unwrap();
        let siteurl = site.url.clone();
        let sitecrawltime = site.crawltime;

        // site shouldn't already exist
        assert!(get_site_by_url(&db, &siteurl).unwrap().is_none());

        db.set_site(site).unwrap();

        // site should exist
        let result = get_site_by_url(&db, &siteurl).unwrap();
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.url, siteurl);
        assert_eq!(result.crawltime, sitecrawltime);
    }

    #[test]
    fn get_site_update() {
        let db = Database::connect_virtual().unwrap();

        let site = create_site("https://dawnvoid.neocities.org/", 0).unwrap();
        let expected = create_site("https://dawnvoid.neocities.org/", 0).unwrap();

        db.set_site(site).unwrap();

        // site should exist
        let result = get_site_by_url(&db, "https://dawnvoid.neocities.org/").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), expected);

        // change it
        let site = create_site("https://dawnvoid.neocities.org/", 999).unwrap();
        let expected = create_site("https://dawnvoid.neocities.org/", 999).unwrap();

        db.set_site(site).unwrap();

        // site should exist and be changed
        let result = get_site_by_url(&db, "https://dawnvoid.neocities.org/").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn delete_site_deletes_links() {
        let db = Database::connect_virtual().unwrap();

        let sites = vec![
            "https://dawnvoid.neocities.org/",
            "https://scarbyte.neocities.org/",
            "https://koyo.neocities.org/",
            "https://errormine.neocities.org/",
            "https://undoified.neocities.org/",
            "https://personally-comfy.neocities.org/",
            "https://bytemoth.neocities.org/",
            "https://jackomix.neocities.org/",
            "https://warningnonpotablewater.neocities.org/",
            "https://kryptonaut.neocities.org/",
            "https://psychicnewborn.neocities.org/",
            "https://omnipresence.neocities.org/",
        ];
        for &s in &sites {
            let site = create_site(s, 0).unwrap();
            db.set_site(site).unwrap();
        }

        let links = vec![
            (
                "https://jackomix.neocities.org/",
                "https://jackomix.neocities.org/",
            ),
            (
                "https://jackomix.neocities.org/",
                "https://koyo.neocities.org/",
            ),
            (
                "https://jackomix.neocities.org/",
                "https://errormine.neocities.org/",
            ),
            (
                "https://jackomix.neocities.org/",
                "https://undoified.neocities.org/",
            ),
            (
                "https://koyo.neocities.org/",
                "https://jackomix.neocities.org/",
            ),
        ];
        for &l in &links {
            let link = create_link(l.0, l.1).unwrap();
            db.set_link(link).unwrap();
        }

        // all links should exist (TODO: stricter checking)
        let dblinks = db
            .get_links_by_srcurl(
                create_link("https://jackomix.neocities.org/", "https://example.org/").unwrap(),
            )
            .unwrap();
        assert_eq!(dblinks.len(), 4);
        let dblinks = db
            .get_links_by_srcurl(
                create_link("https://koyo.neocities.org/", "https://example.org/").unwrap(),
            )
            .unwrap();
        assert_eq!(dblinks.len(), 1);

        // delete a site that links depend on
        db.delete_site_by_url(create_site("https://jackomix.neocities.org/", 0).unwrap())
            .unwrap();
        assert!(get_site_by_url(&db, "https://jackomix.neocities.org/")
            .unwrap()
            .is_none());

        // only dependant links should have been removed
        let dblinks = db
            .get_links_by_srcurl(
                create_link("https://jackomix.neocities.org/", "https://example.org/").unwrap(),
            )
            .unwrap();
        assert_eq!(dblinks.len(), 0);
        let dblinks = db
            .get_links_by_srcurl(
                create_link("https://koyo.neocities.org/", "https://example.org/").unwrap(),
            )
            .unwrap();
        assert_eq!(dblinks.len(), 1);
    }

    #[test]
    pub fn set_link_no_site() {
        let db = Database::connect_virtual().unwrap();

        // Attempting to insert a link with a srcurl that doesn't exist as a url entry in the site table should fail.
        // The srcurl is a foreign key that depends on urls in the site table.
        let link = create_link(
            "https://errormine.neocities.org/",
            "https://scarbyte.neocities.org/",
        )
        .unwrap();
        assert!(db.set_link(link).is_err());
    }
}
