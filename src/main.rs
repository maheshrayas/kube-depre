use clap::Parser;
use kube_depre::Cluster;
use kube_depre::FileSystem;
use kube_depre::{init_logger, Finder, Output, Scrape, VecTableDetails};
use log::error;
use std::process::exit;

const K8_VERSIONS: [&str; 4] = ["1.16", "1.22", "1.25", "1.26"];
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Sunset {
    /// list of deprecated apis in a specific kubernetes version, -t 1.22
    /// if -t not supplied, it will query for versions : 1.16, 1.22, 1.25, 1.26
    #[clap(long = "target-version", short = 't')]
    target_version: Option<String>,
    /// -o csv. Default is table
    /// Output format table, csv
    #[clap(long = "output", short = 'o', arg_enum,default_value_t = Output::Table)]
    output: Output,
    #[clap(long, short)]
    kubeconfig: Option<String>,
    /// supply -f or --file "Manifest file directory".
    /// if -f not supplied, it will by default query the cluster
    #[clap(long, short)]
    file: Option<String>,
    /// supply --debug to print the debug information
    #[clap(short, long, parse(from_occurrences))]
    debug: usize,
}

impl Sunset {
    // if there is a mention of -d in the args, it will be scraping the directory else default will be cluster
    fn check_scrape_type(&self) -> Scrape {
        match &self.file {
            Some(d) => Scrape::Dir(d.to_string(), "Filename"),
            None => Scrape::Cluster("Namespace"),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();
    let cli = Sunset::parse();
    let versions: Vec<String> = if let Some(version) = &cli.target_version {
        if K8_VERSIONS.contains(&version.as_str()) {
            [version.to_string()].to_vec()
        } else {
            error!("Version {} does not have any deprecated apis", version);
            exit(0);
        }
    } else {
        K8_VERSIONS.iter().map(|v| v.to_string()).collect()
    };

    match cli.debug {
        1 => {
            std::env::set_var("RUST_LOG", "info,kube=debug");
        }
        _ => std::env::set_var("RUST_LOG", "info,kube=info"),
    }
    match cli.check_scrape_type() {
        Scrape::Cluster(col_replace) => {
            let c = Cluster::new(versions).await?;
            let x = VecTableDetails(c.find_deprecated_api().await?);
            if !x.0.is_empty() {
                match cli.output {
                    Output::Csv => {
                        x.generate_csv(col_replace)?;
                    }
                    Output::Junit => {
                        println!("Junit");
                    }
                    Output::Table => {
                        x.generate_table(col_replace)?;
                    }
                }
            }
        }
        Scrape::Dir(loc, col_replace) => {
            let c = FileSystem::new(loc, versions).await?;
            let x = VecTableDetails(c.find_deprecated_api().await?);
            if !x.0.is_empty() {
                match cli.output {
                    Output::Csv => {
                        x.generate_csv(col_replace)?;
                    }
                    Output::Junit => {
                        println!("Junit");
                    }
                    Output::Table => {
                        x.generate_table(col_replace)?;
                    }
                }
            }
        }
    };

    Ok(())
}
