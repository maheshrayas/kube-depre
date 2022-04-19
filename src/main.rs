use comfy_table::Table;
use file::search_files;
use utils::{generate_csv_header, Deprecated, Output, Scrape};
mod cluster;
mod file;
mod utils;
use crate::cluster::get_cluster_resources;
use crate::utils::{generate_table_header, init_logger, ClusterOP, VecTableDetails};
use clap::Parser;
use log::info;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Sunset {
    #[clap(long = "target-version", short = 't')]
    target_version: Option<String>,
    /// Output format table, junit, csv
    #[clap(long = "output", short = 'o', arg_enum,default_value_t = Output::Table)]
    output: Output,
    #[clap(long, short)]
    kubeconfig: Option<String>,
    /// Scrape the cluster for deprecated apis,
    #[clap(long, short)]
    file: Option<String>,
    #[clap(short, long, parse(from_occurrences))]
    debug: usize,
}

impl Sunset {
    // if there is a mention of -d in the args, it will be scraping the directory else default will be cluster
    fn check_scrape_type(&self) -> Scrape {
        match &self.file {
            Some(d) => Scrape::Dir(d.to_string()),
            None => Scrape::Cluster,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Sunset::parse();
    // You can check the value provided by positional arguments, or option arguments
    let version = if let Some(version) = cli.target_version.as_deref() {
        version
    } else {
        "1.16"
    };

    match cli.debug {
        1 => {
            std::env::set_var("RUST_LOG", "info,kube=debug");
        }
        _ => std::env::set_var("RUST_LOG", "info,kube=info"),
    }

    init_logger();

    let val = Deprecated::get_apiversion(format!("v{}", version).as_str())
        .await?
        .as_array()
        .unwrap()
        .to_owned();

    match cli.check_scrape_type() {
        Scrape::Cluster => {
            let x = utils::VecTableDetails(get_cluster_resources(version, val).await?);
            match cli.output {
                Output::Csv => {
                    let mut wtr = csv::Writer::from_path("./deprecated-list.csv")?;
                    generate_csv_header(&mut wtr, "Filename")?;
                    x.generate_csv(&mut wtr)?;
                    wtr.flush()?;
                    info!(
                        "deprecated-list.csv written at location {}",
                        std::env::current_dir()?.as_os_str().to_str().unwrap()
                    );
                }
                Output::Junit => {
                    println!("Junit");
                }
                Output::Table => {
                    let mut t = Table::new();
                    let t = generate_table_header(&mut t, "Namespace");
                    x.generate_table(t)?;
                    println!("{t}");
                }
            }
        }
        Scrape::Dir(loc) => {
            let x: VecTableDetails = utils::VecTableDetails(search_files(val, loc));
            match cli.output {
                Output::Csv => {
                    let mut wtr = csv::Writer::from_path("./deprecated-list.csv")?;
                    generate_csv_header(&mut wtr, "Filename")?;
                    x.generate_csv(&mut wtr)?;
                    wtr.flush()?;
                    info!(
                        "deprecated-list.csv written at location {}",
                        std::env::current_dir()?.as_os_str().to_str().unwrap()
                    );
                }
                Output::Junit => {
                    println!("Junit");
                }
                Output::Table => {
                    let mut t = Table::new();
                    let t = generate_table_header(&mut t, "filename");
                    x.generate_table(t)?;
                    println!("{t}");
                }
            }
        }
    }
    Ok(())
}
