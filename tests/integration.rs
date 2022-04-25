use async_trait::async_trait;
use kube_depre::utils::*;
use std::path::Path;

#[tokio::test]
async fn test_kube_deprecated_api() {
    struct Te;
    #[async_trait]
    impl Finder for Te {
        async fn find_deprecated_api(&self) -> anyhow::Result<Vec<TableDetails>> {
            Ok(vec![])
        }
    }
    let x: Vec<String> = vec!["1.22".to_string()];
    let m = Te::get_deprecated_api(x).await;
    assert!(m.is_ok());
}

#[tokio::test]
async fn test_check_all_deprecated_api_version_len() {
    struct Te;
    #[async_trait]
    impl Finder for Te {
        async fn find_deprecated_api(&self) -> anyhow::Result<Vec<TableDetails>> {
            Ok(vec![])
        }
    }
    let x: Vec<String> = ["1.16", "1.22", "1.25", "1.26"]
        .iter()
        .map(|v| v.to_string())
        .collect();
    let m = Te::get_deprecated_api(x).await;
    assert_eq!(m.unwrap().len(), 35);
}

#[tokio::test]
async fn test_table_generation() {
    let t = TableDetails {
        kind: "ValidatingWebhookConfiguration".to_string(),
        namespace: "".to_string(),
        name: "istiod-istio-system".to_string(),
        deprecated_api_version: "admissionregistration.k8s.io/v1beta1".to_string(),
        supported_api_version: "admissionregistration.k8s.io/v1".to_string(),
        k8_version: "1.22".to_string(),
    };
    let table = VecTableDetails(vec![t]);
    let x = table.generate_table("Namespace");
    assert!(x.is_ok());
}

#[tokio::test]
async fn test_csv_generation() {
    let t = TableDetails {
        kind: "ValidatingWebhookConfiguration".to_string(),
        namespace: "".to_string(),
        name: "istiod-istio-system".to_string(),
        deprecated_api_version: "admissionregistration.k8s.io/v1beta1".to_string(),
        supported_api_version: "admissionregistration.k8s.io/v1".to_string(),
        k8_version: "1.22".to_string(),
    };
    let table = VecTableDetails(vec![t]);
    let _ = table.generate_csv("Filename");
    let x = Path::new("./deprecated-list.csv").exists();
    assert_eq!(x, true);
}
