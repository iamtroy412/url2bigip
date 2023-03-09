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
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    info!("Parsing command-line arguments");
    let args = Args::parse();

    debug!("`&args.input_file`: {:?}", &args.input_file);

    let mut prom_out = url2bigip::Prom {
        targets: url2bigip::build_urls(&args.input_file)?,
        labels: HashMap::from([
            ("location".to_owned(), "BigIP".to_owned())
        ]),
    };
    debug!("Prometheus JSON:\n{}", serde_json::to_string_pretty(&prom_out).unwrap());
    
    Ok(())
}