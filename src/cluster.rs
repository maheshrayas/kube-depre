use crate::utils::{Deprecated, JsonDetails, TableDetails};
use anyhow::Result;
use kube::{
    api::{Api, DynamicObject, ResourceExt},
    core::GroupVersionKind,
    discovery::pinned_kind,
    Client,
};
use log::info;
use std::sync::Arc;
use tokio::task::spawn;

pub(crate) async fn get_cluster_resources(
    version: &str,
) -> Result<Vec<tokio::task::JoinHandle<Result<Vec<TableDetails>>>>> {
    let client = Client::try_default().await?;
    //let current_config = kube::config::Config::infer().await?;
    let current_config = kube::config::Kubeconfig::read().unwrap();
    info!(
        "Connected to cluster {:?}",
        current_config.current_context.unwrap()
    );
    info!("Target apiversions v{}", version);

    let val = Deprecated::get_apiversion(format!("v{}", version).as_str())
        .await?
        .as_array()
        .unwrap()
        .to_owned();
    Ok(val
        .into_iter()
        .map(|resource| {
            let arc_client = Arc::new(client.clone());
            let client_clone = Arc::clone(&arc_client);
            spawn(async move {
                let mut temp_table: Vec<TableDetails> = vec![];
                let kind = resource["kind"].as_str().unwrap().to_string();
                let group = resource["group"].as_str().unwrap().to_string();
                let version = resource["version"].as_str().unwrap().to_string();
                let removed = resource["removed"].as_str().unwrap().to_string();
                let gvk = GroupVersionKind::gvk(&group, &version, &kind);
                let (ar, _) = pinned_kind(&(*client_clone).clone(), &gvk).await?;
                let api: Api<DynamicObject> = Api::all_with((*client_clone).clone(), &ar);
                let list = if let Ok(list) = api.list(&Default::default()).await {
                    list
                } else {
                    return Ok(temp_table);
                };

                for item in list.items {
                    let name = item.name();
                    let ns = item.to_owned().metadata.namespace.unwrap_or_default();
                    if removed.eq("true") {
                        temp_table.push(TableDetails {
                            kind: ar.kind.to_string(),
                            namespace: ns,
                            name,
                            supported_api_version: "REMOVED".to_string(),
                            deprecated_api_version: "REMOVED".to_string(),
                        });
                    } else {
                        let annotations = item.annotations();
                        let last_applied_apiversion = if let Some(last_applied) =
                            annotations.get("kubectl.kubernetes.io/last-applied-configuration")
                        {
                            let m: JsonDetails = serde_json::from_str(last_applied)?;
                            Some(m.api_version)
                        } else {
                            None
                        };

                        if let Some(ls_app_ver) = last_applied_apiversion {
                            let supported_version = format!("{}/{}", &group, &version);
                            if !ls_app_ver.eq(&supported_version) {
                                let t = TableDetails {
                                    kind: ar.kind.to_string(),
                                    namespace: ns,
                                    name,
                                    supported_api_version: supported_version,
                                    deprecated_api_version: ls_app_ver,
                                };
                                temp_table.push(t);
                            }
                        }
                    }
                }

                Ok(temp_table)
            })
        })
        .collect())
}
