use anyhow::Result;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct JsonDetails {
    #[serde(rename = "apiVersion")]
    pub(crate) api_version: String,
}

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
    pub(crate) async fn get_apiversion(version: &str) -> Result<Self> {
        let url = format!(
            "https://raw.githubusercontent.com/maheshrayas/k8s_deprecated_api/main/{}/data.json",
            version
        );
        let x = reqwest::get(url).await?.json::<Self>().await?;
        Ok(x)
    }
}
