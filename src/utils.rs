use anyhow::Result;
use clap::ArgEnum;
use comfy_table::{ContentArrangement, Table};
use csv::Writer;
use env_logger::{Builder, Env};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs::File, io::Write};
use tokio::task::JoinHandle;

pub(crate) type ClusterOP = Vec<JoinHandle<Result<Vec<TableDetails>>>>;

#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct JsonDetails {
    #[serde(rename = "apiVersion")]
    pub(crate) api_version: String,
}

pub(crate) struct VecTableDetails(pub(crate) Vec<TableDetails>);

#[derive(Default)]
pub(crate) struct TableDetails {
    pub(crate) kind: String,
    pub(crate) namespace: String,
    pub(crate) name: String,
    pub(crate) deprecated_api_version: String,
    pub(crate) supported_api_version: String,
}

impl VecTableDetails {
    pub(crate) fn generate_table(self, t: &mut Table) -> Result<()> {
        for r in self.0 {
            t.add_row(vec![
                r.kind,
                r.namespace,
                r.name,
                r.deprecated_api_version,
                r.supported_api_version,
            ]);
        }
        Ok(())
    }
    pub(crate) fn generate_csv(self, wtr: &mut Writer<File>) -> Result<()> {
        for r in self.0 {
            wtr.write_record([
                r.kind,
                r.namespace,
                r.name,
                r.deprecated_api_version,
                r.supported_api_version,
            ])?;
        }
        Ok(())
    }
}

pub(crate) fn generate_table_header<'a>(t: &'a mut Table, column_replace: &str) -> &'a mut Table {
    t.set_header(vec![
        "Kind",
        column_replace,
        "Name",
        "DeprecatedApiVersion",
        "SupportedApiVersion",
    ])
    .set_content_arrangement(ContentArrangement::Dynamic)
}

pub(crate) fn generate_csv_header(wtr: &mut Writer<File>, column_replace: &str) -> Result<()> {
    wtr.write_record([
        "Kind",
        column_replace,
        "Name",
        "DeprecatedApiVersion",
        "SupportedApiVersion",
    ])?;
    Ok(())
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct Deprecated {
    pub(crate) apis: serde_json::Value,
}

impl Deprecated {
    pub(crate) async fn get_apiversion(version: &str) -> Result<Value> {
        let url = format!(
            "https://raw.githubusercontent.com/maheshrayas/k8s_deprecated_api/main/{}/data.json",
            version
        );
        let x: Value = reqwest::get(url).await?.json().await?;
        Ok(x)
    }
}

pub(crate) fn init_logger() {
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
pub(crate) enum Output {
    Table,
    Junit,
    Csv,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Scrape {
    Cluster,
    Dir(String),
}
