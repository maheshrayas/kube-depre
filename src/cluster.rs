use crate::utils::{ClusterOP, DepreApi, Finder, JsonDetails, TableDetails};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use kube::{
    api::{Api, DynamicObject, ResourceExt},
    core::GroupVersionKind,
    discovery::pinned_kind,
    Client,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task::spawn;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Cluster {
    deprecated_api_result: Vec<DepreApi>,
}

impl<'a> Cluster {
    pub async fn new(version: Vec<&str>) -> anyhow::Result<Cluster> {
        let deprecated_api_result = Self::get_deprecated_api(version).await?;
        if !deprecated_api_result.is_empty() {
            Ok(Cluster {
                deprecated_api_result,
            })
        } else {
            Err(anyhow!(
                "Input target version does not have any kubernetes deprecated APIs"
            ))
        }
    }
}

#[async_trait]
impl Finder for Cluster {
    async fn find_deprecated_api(&self) -> Result<Vec<TableDetails>> {
        let client = Client::try_default().await?;
        let current_config = kube::config::Kubeconfig::read().unwrap();
        info!(
            "{} Connected to cluster {:?}",
            String::from("\u{2638}"),
            current_config.current_context.unwrap()
        );
        let m = self.deprecated_api_result.to_owned();
        let join_handle: ClusterOP = m
            .into_iter()
            .map(|resource| {
                let arc_client = Arc::new(client.clone());
                let client_clone = Arc::clone(&arc_client);
                spawn(async move {
                    let mut temp_table: Vec<TableDetails> = vec![];
                    let kind = resource.kind.to_string();
                    let group = resource.group.to_string();
                    let version = resource.version.to_string();
                    let removed = resource.removed.to_string();
                    let k8_version = resource.k8_version.to_owned().unwrap().to_string();
                    let deprecated_version = resource.deprecated_versions.to_owned();
                    let gvk = GroupVersionKind::gvk(&group, &version, &kind);
                    let ar = match pinned_kind(&(*client_clone).clone(), &gvk).await {
                        Ok((ar, _)) => ar,
                        Err(e) => match e {
                            // this error may occur when you donot have the object in cluster, so don't panic, rather return empty string
                            kube::Error::Api(_) => return Ok(temp_table),
                            _ => return Err(anyhow!(e)),
                        },
                    };
                    let api: Api<DynamicObject> = Api::all_with((*client_clone).clone(), &ar);
                    let list = if let Ok(list) = api.list(&Default::default()).await {
                        list
                    } else {
                        return Ok(temp_table);
                    };

                    for item in list.items {
                        let name = item.to_owned().metadata.name.unwrap_or_default();
                        let ns = item.to_owned().metadata.namespace.unwrap_or_default();
                        if removed.eq("true") {
                            temp_table.push(TableDetails {
                                kind: ar.kind.to_string(),
                                namespace: ns,
                                name,
                                supported_api_version: "REMOVED".to_string(),
                                deprecated_api_version: "REMOVED".to_string(),
                                k8_version: k8_version.to_string(),
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
                                if deprecated_version.is_some() {
                                    // this check if for when 2 different versions are supported in kubernetes
                                    // for example, autoscaling/v2 and autoscaling/v1 are both supported
                                    println!("I am here {}", ls_app_ver);
                                    if deprecated_version.as_ref().unwrap().contains(&ls_app_ver) {
                                        temp_table.push(TableDetails {
                                            kind: ar.kind.to_string(),
                                            namespace: ns,
                                            name,
                                            supported_api_version: supported_version,
                                            deprecated_api_version: ls_app_ver,
                                            k8_version: k8_version.to_string(),
                                        });
                                    }
                                } else {
                                    if !ls_app_ver.eq(&supported_version) {
                                        temp_table.push(TableDetails {
                                            kind: ar.kind.to_string(),
                                            namespace: ns,
                                            name,
                                            supported_api_version: supported_version,
                                            deprecated_api_version: ls_app_ver,
                                            k8_version: k8_version.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    Ok(temp_table)
                })
            })
            .collect();
        let mut v: Vec<TableDetails> = vec![];
        for task in join_handle {
            v.append(&mut task.await??);
        }
        Ok(v)
    }
}
