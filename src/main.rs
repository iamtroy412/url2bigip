use anyhow::{Context, Result};
use assert_fs::prelude::*;
use std::io::{prelude::*, BufReader};
use log::{debug, info, warn};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use url::{Url, ParseError};

#[derive(Debug, Serialize)]
pub struct Prom {
    pub targets: Vec<Url>,
    pub labels: HashMap<String, String>,
}

pub fn build_urls(path: &Path) -> Result<Vec<Url>, anyhow::Error> {
    // TODO
    info!("Opening `{}` for reading...", path.display());
    let file = File::open(&path).with_context(|| format!("Failed to open `{}`!", path.display()))?;

    let reader = BufReader::new(file);

    let mut urls = Vec::new();

    info!("Reading lines from `{}` into vector of URLs...", path.display());
    for line in reader.lines() {
        let line = line.with_context(||
            format!("Failed to read line from `{}`!", path.display()))?;
        
        debug!("`&line`: {:?}", line);
        match Url::parse(&line) {
            Ok(url) => {
                urls.push(url);
            }
            Err(e) => {
                // TODO: Either write URLs that didn't parse to file, or
                // something else to keep track of them after the program runs.
                warn!("Failed to parse url `{}`! Error: {}", &line, e);
            }
        }
    }

    info!("Read {} URLs from `{}`", urls.len(), path.display());
    debug!("`&urls`: {:?}", urls);
    Ok(urls)

}



#[test]
fn test_build_urls () {
    let file = assert_fs::NamedTempFile::new("sample.txt").unwrap();
    file.write_str("google.com\nhttps://google.com\nasfasdf.asdf\nyahoo.com\nhttp://espn.com").unwrap();

    let base_case =
        Prom {
            targets: vec![
                Url::parse("https://google.com").unwrap(),
                Url::parse("http://espn.com").unwrap(),
            ],
            labels: HashMap::from([
                ("location".to_owned(), "BigIP".to_owned())
            ]),
        };

    let result = build_urls(&file.path()).unwrap();

    assert_eq!(base_case.targets[0].as_str(), result[0].as_str());
    assert_eq!(base_case.targets[1].as_str(), result[1].as_str());
}