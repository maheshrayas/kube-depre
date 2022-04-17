use anyhow::Result;
use clap::ArgEnum;
use comfy_table::Table;
use env_logger::{Builder, Env};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use tokio::task::JoinHandle;

pub(crate) type ClusterOP = Vec<JoinHandle<Result<Vec<TableDetails>>>>;

pub(crate) struct DeprecatedResult {
    pub(crate) output: ClusterOP,
}

impl DeprecatedResult {
    pub(crate) fn new(d: ClusterOP) -> Self {
        Self { output: d }
    }

    pub(crate) async fn generate_table(self) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            "Kind",
            "Namespace",
            "Name",
            "DeprecatedApiVersion",
            "SupportedApiVersion",
        ]);
        for task in self.output {
            let result = task.await?.unwrap();
            for r in result {
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

    pub(crate) async fn generate_csv(self) -> Result<()> {
        let mut wtr = csv::Writer::from_path("./deprecated-list.csv")?;
        wtr.write_record([
            "Kind",
            "Namespace",
            "Name",
            "DeprecatedApiVersion",
            "SupportedApiVersion",
        ])?;
        for task in self.output {
            let result = task.await?.unwrap();
            for r in result {
                wtr.write_record([
                    r.kind,
                    r.namespace,
                    r.name,
                    r.deprecated_api_version,
                    r.supported_api_version,
                ])?;
            }
        }
        wtr.flush()?;
        info!(
            "deprecated-list.csv written at location {}",
            std::env::current_dir()?.as_os_str().to_str().unwrap()
        );
        Ok(())
    }
}
#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct JsonDetails {
    #[serde(rename = "apiVersion")]
    pub(crate) api_version: String,
}

#[derive(Default)]
pub(crate) struct TableDetails {
    pub(crate) kind: String,
    pub(crate) namespace: String,
    pub(crate) name: String,
    pub(crate) deprecated_api_version: String,
    pub(crate) supported_api_version: String,
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
