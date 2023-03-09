use anyhow::{Context, Result};
use assert_fs::prelude::*;
use dns_lookup::lookup_host;
use ipnet::Ipv4Net;
use log::{debug, info, warn};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{prelude::*, BufReader};
use std::fs::File;
use std::path::Path;
use std::net::{IpAddr, Ipv4Addr};
use url::{Url, ParseError};

#[derive(Debug, Serialize)]
pub struct Prom {
    pub targets: Vec<Url>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Site {
    pub url: Url,
    pub ips: Vec<IpAddr>,
}

pub fn build_urls(path: &Path) -> Result<Vec<Url>, anyhow::Error> {
    // TODO
    info!("Opening `{}` for reading...", path.display());
    let file = File::open(path).with_context(|| format!("Failed to open `{}`!", path.display()))?;

    let reader = BufReader::new(file);

    let mut urls = Vec::new();

    info!("Reading lines from `{}` into vector of URLs...", path.display());
    for line in reader.lines() {
        let line = line.with_context(||
            format!("Failed to read line from `{}`!", path.display()))?;
        
        debug!("`&line`: {:?}", line);
        match Url::parse(&line) {
            Ok(url) => {
                debug!("`&url` as str: {:?}", url.as_str());
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
    file.write_str("google.com\nhttps://google.com\nasfasdf.asdf\nyahoo.com\nhttps://*.blah.pitt.edu\nhttp://espn.com").unwrap();

    let base_case = vec![
        Url::parse("https://google.com").unwrap(),
        Url::parse("https://*.blah.pitt.edu").unwrap(),
        Url::parse("http://espn.com").unwrap(),
    ];

    let result = build_urls(&file.path()).unwrap();

    assert_eq!(base_case[0].as_str(), result[0].as_str());
    assert_eq!(base_case[1].as_str(), result[1].as_str());
    assert_eq!(base_case[2].as_str(), result[2].as_str());
}

pub fn lookup_url(urls: &[Url]) -> Vec<Site> {
    let mut sites = Vec::new();

    for url in urls.iter() {
        if let Some(domain) = url.domain() {
            match lookup_host(domain) {
                Ok(addrs) => {
                    debug!("`&addrs`: {:?}", addrs);
                    // The clone here doesn't seem the best? Will return to this...
                    sites.push(Site { url: url.clone(), ips: addrs });
                },
                Err(e) => {
                    // TODO: Either write domains that didn't resolve to file,
                    // or something else to keep track of them after the program runs.
                    warn!("Failed to lookup `{}`! Error: {}", domain, e);
                }
            }
        }
    }

    info!("Found {} sites", sites.len());
    debug!("`sites`: {:?}", sites);
    sites
}

#[test]
fn test_lookup_url () {
    // TODO
    let urls = vec![
        Url::parse("https://google.com").unwrap(),
        Url::parse("https://*.blah.pitt.edu").unwrap(),
        Url::parse("http://espn.com").unwrap(),
    ];

    let results = lookup_url(&urls);
    assert!(results.len() > 0);
    assert!(results[0].ips.len() > 0);
}

pub fn build_subnets(input_path: &Path) -> Result<Vec<Ipv4Net>, anyhow::Error> {
    info!("Opening `{}` for reading", &input_path.display());
    let file = File::open(input_path).with_context(||
        format!("Failed to open `{}`", &input_path.display()))?;
    
    let reader = BufReader::new(file);

    let mut subnets: Vec<Ipv4Net> = Vec::new();

    info!("Reading lines from `{}` into `subnets` vec", &input_path.display());
    for line in reader.lines() {
        let line = line.with_context(||
            format!("Failed to read `{}`", &input_path.display()))?;

        debug!("`&line`: {:?}", &line);
        match &line.parse::<Ipv4Net>() {
            Ok(net) => subnets.push(*net),
            Err(e) => { warn!("{}", e); }
        }
    }
    info!("Read {} subnets from `{}`", subnets.len(), &input_path.display());
    debug!("`subnets`: {:?}", &subnets);
    Ok(subnets)
}

#[test]
fn test_build_subnets() {
    let file = assert_fs::NamedTempFile::new("sample.txt").unwrap();
    file.write_str("192.168.0.0/24\nnonsense\n172.16.0.0/24\n10.0.0.0/8").unwrap();

    let result = build_subnets(&file.path().to_path_buf()).unwrap();
    assert_eq!(Ipv4Net::new(Ipv4Addr::new(192, 168, 0, 0), 24).unwrap(), result[0]);
    assert_eq!(Ipv4Net::new(Ipv4Addr::new(172, 16, 0, 0), 24).unwrap(), result[1]);
}

pub fn split_targets(sites: &[Site], subnets: &[Ipv4Net]) -> (Vec<Url>, Vec<Url>) {
    // TODO
    let mut bigip_targets = Vec::new();
    let mut other_targets = Vec::new();

    for site in sites.iter() {
        let bigip_match = || {
            for ip in site.ips.iter() {
                for subnet in subnets.iter() {
                    if let IpAddr::V4(v4) = ip {
                        if subnet.contains(v4) {
                            return true
                        }
                    }
                }
            }
            false
        };
        if bigip_match() {
            bigip_targets.push(site.url.clone());
        } else {
            other_targets.push(site.url.clone());
        }
    }

    info!("Found {} bigip targets", bigip_targets.len());
    info!("Found {} other targets", other_targets.len());
    (bigip_targets, other_targets)
}

#[test]
fn test_split_targets() {
    // TODO
}