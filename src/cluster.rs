use crate::utils::{Deprecated, JsonDetails, TableDetails};
use anyhow::Result;
use kube::{
    api::{Api, DynamicObject, ResourceExt},
    discovery::{verbs, ApiGroup, Discovery, Scope},
    Client,
};
use std::sync::Arc;
use tokio::task::spawn;

pub(crate) async fn get_cluster_resources(
    version: &str,
) -> Result<Vec<tokio::task::JoinHandle<Result<Option<TableDetails>>>>> {
    let client = Client::try_default().await?;
    let ns_filter = Arc::new(std::env::var("NAMESPACE").ok());
    let discovery = Discovery::new(client.clone()).run().await?;
    let val = Deprecated::get_apiversion(format!("v{}", version).as_str()).await?;
    let group: Vec<&ApiGroup> = discovery.groups().collect();
    let m = group
        .iter()
        .flat_map(|f| f.recommended_resources())
        .filter_map(|(ar, caps)| {
            val.apis
                .as_object()
                .unwrap()
                .get(&ar.kind)
                .map(|updated_api_vesion| (ar, caps, updated_api_vesion.to_string()))
        });

    Ok(m.into_iter()
        .map(|(ar, caps, updated_api)| {
            let ns_filter_clone = Arc::clone(&ns_filter);
            let arc_client = Arc::new(client.clone());
            let client_clone = Arc::clone(&arc_client);
            spawn(async move {
                // println!("Thread id {:?}", thread::current().id());
                if caps.supports_operation(verbs::LIST) {
                    let api: Api<DynamicObject> = if caps.scope == Scope::Namespaced {
                        if let Some(ns) = &*ns_filter_clone {
                            Api::namespaced_with((*client_clone).clone(), ns, &ar)
                        } else {
                            Api::all_with((*client_clone).clone(), &ar)
                        }
                    } else {
                        Api::all_with((*client_clone).clone(), &ar)
                    };
                    let list = api.list(&Default::default()).await?;
                    for item in list.items {
                        let name = item.name();
                        let ns = item
                            .to_owned()
                            .metadata
                            .namespace
                            .map(|s| s + "/")
                            .unwrap_or_default();
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
                            if !ls_app_ver.eq(&updated_api) {
                                let t = TableDetails {
                                    kind: ar.kind.to_string(),
                                    namespace: ns,
                                    name,
                                    supported_api_version: updated_api.to_string(),
                                    deprecated_api_version: ls_app_ver,
                                };
                                return Ok(Some(t));
                            }
                        }
                    }
                }
                Ok(None)
            })
        })
        .collect())
}
