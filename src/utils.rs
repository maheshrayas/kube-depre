use anyhow::Result;
use clap::ArgEnum;
use comfy_table::{ContentArrangement, Table};
use csv::Writer;
use env_logger::{Builder, Env};
use log::info;

use async_trait::async_trait;
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
    pub(crate) fn generate_table(self, column_replace: &str) -> Result<()> {
        let mut t = Table::new();
        let t = generate_table_header(&mut t, column_replace);
        for r in self.0 {
            t.add_row(vec![
                r.kind,
                r.namespace,
                r.name,
                r.deprecated_api_version,
                r.supported_api_version,
            ]);
        }
        println!("{t}");
        Ok(())
    }
    pub(crate) fn generate_csv(self, column_replace: &str) -> Result<()> {
        let mut wtr = csv::Writer::from_path("./deprecated-list.csv")?;
        generate_csv_header(&mut wtr, column_replace)?;
        for r in self.0 {
            wtr.write_record([
                r.kind,
                r.namespace,
                r.name,
                r.deprecated_api_version,
                r.supported_api_version,
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
pub(crate) enum Scrape<'a> {
    Cluster(&'a str),
    Dir(String, &'a str),
}

#[async_trait]
pub(crate) trait Finder {
    async fn find_deprecated_api(&self) -> Result<Vec<TableDetails>>;
    async fn get_deprecated_api(version: &String) -> anyhow::Result<Vec<Value>> {
        let url = format!(
            "https://raw.githubusercontent.com/maheshrayas/k8s_deprecated_api/main/v{}/data.json",
            version
        );
        let v: Value = reqwest::get(url).await?.json().await?;
        Ok(v.as_array().unwrap().to_owned())
    }
}
