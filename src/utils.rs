use anyhow::Result;
use env_logger::{Builder, Env};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
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
