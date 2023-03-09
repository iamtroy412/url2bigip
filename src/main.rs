use clap::Parser;
use log::{info, debug};
use std::collections::HashMap;
use std::path::PathBuf;

/// A program for parsing a list of URLs and doing DNS queries. 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File with list of URLs to parse.
    #[arg(short, long)]
    input_file: PathBuf,
    /// File with list of subnets to check against
    #[arg(short, long)]
    subnet_file: PathBuf,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    info!("Parsing command-line arguments");
    let args = Args::parse();

    debug!("`&args.input_file`: {:?}", &args.input_file);
    debug!("`&args.subnet_file`: {:?}", &args.subnet_file);

    // Build up a default Prom struct, with empty targets
    // and our pre-defined list of labels.
    // This will contain the list of targets that are on the BigIP
    let mut bigip_prom = url2bigip::Prom {
        targets: Vec::new(),
        labels: HashMap::from([
            ("location".to_owned(), "BigIP".to_owned())
        ]),
    };

    // This will contain the list of targets that are still valid,
    // but NOT on the BigIP.
    let mut other_prom = url2bigip::Prom {
        targets: Vec::new(),
        labels: HashMap::new(),
    };

    // Parse our list of URLs.
    let urls = url2bigip::build_urls(&args.input_file)?;
    // Trim our list of URLs to only those that resolve in DNS.
    let sites = url2bigip::lookup_url(&urls);

    // List of subnets to check IPs against for BigIP ranges.
    let subnets = url2bigip::build_subnets(&args.subnet_file)?;

    (bigip_prom.targets, other_prom.targets) = url2bigip::split_targets(&sites, &subnets);

    // Debug print our resulting JSON files.
    // One for sites with the BigIP label, one for sites without.
    debug!("Prometheus BigIP JSON:\n{}", serde_json::to_string_pretty(&bigip_prom).unwrap());
    debug!("Prometheus Other JSON:\n{}", serde_json::to_string_pretty(&other_prom).unwrap());
    
    Ok(())
}