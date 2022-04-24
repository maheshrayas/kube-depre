use anyhow::Result;
use clap::ArgEnum;
use comfy_table::{ContentArrangement, Table};
use csv::Writer;
use env_logger::{Builder, Env};
use log::{debug, info};

use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write};
use tokio::task::JoinHandle;

pub type ClusterOP = Vec<JoinHandle<Result<Vec<TableDetails>>>>;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct JsonDetails {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
}

pub struct VecTableDetails(pub Vec<TableDetails>);

#[derive(Default)]
pub struct TableDetails {
    pub kind: String,
    pub namespace: String,
    pub name: String,
    pub deprecated_api_version: String,
    pub supported_api_version: String,
    pub k8_version: String,
}

impl VecTableDetails {
    pub fn generate_table(self, column_replace: &str) -> Result<()> {
        let mut t = Table::new();
        let t = generate_table_header(&mut t, column_replace);
        for r in self.0 {
            t.add_row(vec![
                r.kind,
                r.namespace,
                r.name,
                r.deprecated_api_version,
                r.supported_api_version,
                r.k8_version,
            ]);
        }
        println!("{t}");
        Ok(())
    }
    pub fn generate_csv(self, column_replace: &str) -> Result<()> {
        let mut wtr = csv::Writer::from_path("./deprecated-list.csv")?;
        generate_csv_header(&mut wtr, column_replace)?;
        for r in self.0 {
            wtr.write_record([
                r.kind,
                r.namespace,
                r.name,
                r.deprecated_api_version,
                r.supported_api_version,
                r.k8_version,
            ])?;
        }
        wtr.flush()?;
        info!(
            "deprecated-list.csv written at location {}",
            std::env::current_dir()?.as_os_str().to_str().unwrap()
        );
        Ok(())
    }
}

pub fn generate_table_header<'a>(t: &'a mut Table, column_replace: &str) -> &'a mut Table {
    t.set_header(vec![
        "Kind",
        column_replace,
        "Name",
        "DeprecatedApiVersion",
        "SupportedApiVersion",
        "K8sVersion",
    ])
    .set_content_arrangement(ContentArrangement::Dynamic)
}

pub fn generate_csv_header(wtr: &mut Writer<File>, column_replace: &str) -> Result<()> {
    wtr.write_record([
        "Kind",
        column_replace,
        "Name",
        "DeprecatedApiVersion",
        "SupportedApiVersion",
        "K8sVersion",
    ])?;
    Ok(())
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Deprecated {
    pub apis: serde_json::Value,
}

pub fn init_logger() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    let env = Env::default()
        .filter("RUST_LOG")
        .write_style("MY_LOG_STYLE");

    Builder::from_env(env)
        .format(|buf, record| {
            let style = buf.style();
            // style.set_bg(Color::Yellow).set_bold(true);

            let timestamp = buf.timestamp();

            writeln!(buf, "{}: {}", timestamp, style.value(record.args()))
        })
        .init();
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum Output {
    Table,
    Junit,
    Csv,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scrape<'a> {
    Cluster(&'a str),
    Dir(String, &'a str),
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct DepreApi {
    pub kind: String,
    pub group: String,
    pub version: String,
    pub removed: String,
    pub k8_version: Option<String>,
}

#[async_trait]
pub trait Finder {
    async fn find_deprecated_api(&self) -> Result<Vec<TableDetails>>;
    async fn get_deprecated_api(versions: Vec<String>) -> anyhow::Result<Vec<DepreApi>> {
        //let mut apis: Vec<Value> = vec![];
        let mut output: Vec<DepreApi> = vec![];
        for version in versions {
            info!(
                "Getting list of deperecated apis in kubernetes version {}",
                version
            );
            let url = format!(
            "https://raw.githubusercontent.com/maheshrayas/k8s_deprecated_api/main/v{}/data.json",
            version
        );
            debug!("deprecated list url {}", url);
            let v: Vec<DepreApi> = reqwest::get(url).await?.json().await?;
            for mut k in v {
                k.k8_version = Some(version.to_owned());
                output.push(k)
            }
        }
        Ok(output)
    }
}
