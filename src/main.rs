use utils::{DeprecatedResult, Output};
mod cluster;
mod utils;
use crate::cluster::get_cluster_resources;
use crate::utils::{init_logger, ClusterOP};
use clap::Parser;

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
    #[clap(short, long, parse(from_occurrences))]
    debug: usize,
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

    let join_handle: ClusterOP = get_cluster_resources(version).await?;
    let d = DeprecatedResult::new(join_handle);

    match cli.output {
        Output::Csv => {
            d.generate_csv().await?;
        }
        Output::Junit => {
            println!("Junit");
        }
        Output::Table => {
            d.generate_table().await?;
        }
    }
    Ok(())
}
