use anyhow::Result;
use comfy_table::Table;
mod cluster;
mod utils;
use crate::cluster::get_cluster_resources;
use crate::utils::TableDetails;
use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Sunset {
    #[clap(long = "target-version", short = 't')]
    target_version: Option<String>,
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

    env_logger::init();

    let mut table = Table::new();
    table.set_header(vec![
        "Kind",
        "Namespace",
        "Name",
        "DeprecatedApiVersion",
        "SupportedApiVersion",
    ]);

    let join_handle: Vec<tokio::task::JoinHandle<Result<Option<TableDetails>>>> =
        get_cluster_resources(version).await?;

    for task in join_handle {
        let result = task.await?.unwrap();
        if let Some(r) = result {
            table.add_row(vec![
                r.kind,
                r.namespace,
                r.name,
                r.deprecated_api_version,
                r.supported_api_version,
            ]);
        }
    }
    println!("{table}");
    Ok(())
}
