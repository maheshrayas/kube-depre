use clap::Parser;
use kube_depre::Cluster;
use kube_depre::FileSystem;
use kube_depre::K8_VERSIONS;
use kube_depre::{init_logger, Finder, Output, Scrape};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Sunset {
    /// list of deprecated APIs in a specific kubernetes version, -t 1.22.
    /// If -t not supplied, it will query for versions : 1.16, 1.22, 1.25, 1.26
    #[clap(long = "target-version", short = 't')]
    target_version: Option<String>,
    /// Output format for the list of deprecated APIs.
    #[clap(long = "output", short = 'o', arg_enum,default_value_t = Output::Table)]
    output: Output,
    /// location of Kubeconfig, Default: ~/.kube/config
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
    let versions: Vec<&str> = if let Some(version) = &cli.target_version {
        version.as_str().split(",").collect::<Vec<&str>>()
    } else {
        K8_VERSIONS.iter().map(|v| *v).collect()
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
            c.process(cli.output, col_replace).await?;
        }
        Scrape::Dir(loc, col_replace) => {
            let f = FileSystem::new(loc.to_owned(), versions).await?;
            f.process(cli.output, col_replace).await?;
        }
    };
    Ok(())
}
