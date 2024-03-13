use rusqlite::{self, Connection, OptionalExtension, Statement};
use std::path::Path;
use url::Url;

#[derive(Debug)]
pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn connect(path: &Path) -> Result<Database, String> {
        // let con = Connection::open_in_memory()?;
        let con = Connection::open(path).map_err(|e| e.to_string())?;
        let d = Database { connection: con };
        SiteEntry::create_table(&d.connection)?;
        LinkEntry::create_table(&d.connection)?;
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

    pub fn insert_new_link(&self, link: LinkEntry) -> Result<(), String> {
        link.insert_new(&self.connection)?;
        Ok(())
    }

    pub fn delete_link_by_srcurl(&self, link: LinkEntry) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM link WHERE srcurl = ?1", (link.srcurl,))
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

    // pub fn get_site_by_
}

#[derive(Debug)]
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

    fn create_table(con: &Connection) -> Result<(), String> {
        con.execute(
            "CREATE TABLE IF NOT EXISTS site (
                url TEXT NOT NULL PRIMARY KEY,
                crawltime INTEGER NOT NULL
            )",
            (),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn insert_new(self, con: &Connection) -> Result<(), String> {
        con.execute(
            "INSERT INTO site (url, crawltime) VALUES (?1, ?2)",
            (self.url, self.crawltime),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct LinkEntry {
    id: i64,        // primary key
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
            id: 0,
            srcurl: String::from(srcurl.as_str()),
            dsturl: String::from(dsturl.as_str()),
        };
        Ok(l)
    }

    fn create_table(con: &Connection) -> Result<(), String> {
        con.execute(
            "CREATE TABLE IF NOT EXISTS link (
                id     INTEGER PRIMARY KEY,
                srcurl TEXT NOT NULL,
                dsturl TEXT NOT NULL,
                FOREIGN KEY (srcurl) REFERENCES site (url)
                    ON UPDATE CASCADE
                    ON DELETE CASCADE
            )",
            (),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn insert_new(self, con: &Connection) -> Result<(), String> {
        con.execute(
            "INSERT INTO link (srcurl, dsturl) VALUES (?1, ?2)",
            (self.srcurl, self.dsturl),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
