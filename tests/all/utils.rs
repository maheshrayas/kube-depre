use async_trait::async_trait;
use kube_depre::{Finder, Output, TableDetails, VecTableDetails};
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
    let x: Vec<&str> = vec!["1.22"];
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
    let x: Vec<&str> = ["1.16", "1.22", "1.25", "1.26"]
        .iter()
        .map(|v| *v)
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

#[tokio::test]
async fn test_markdown_table_generation() {
    let t = TableDetails {
        kind: "ValidatingWebhookConfiguration".to_string(),
        namespace: "".to_string(),
        name: "istiod-istio-system".to_string(),
        deprecated_api_version: "admissionregistration.k8s.io/v1beta1".to_string(),
        supported_api_version: "admissionregistration.k8s.io/v1".to_string(),
        k8_version: "1.22".to_string(),
    };
    let table = VecTableDetails(vec![t]);
    let x = table.generate_markdown_table("Namespace");
    assert!(x.is_ok());
}

#[tokio::test]
async fn test_process_generate_table() {
    struct Te;
    #[async_trait]
    impl Finder for Te {
        async fn find_deprecated_api(&self) -> anyhow::Result<Vec<TableDetails>> {
            let m = TableDetails {
                kind: "Kind".to_string(),
                namespace: "Test".to_string(),
                name: "Name".to_string(),
                supported_api_version: "REMOVED".to_string(),
                deprecated_api_version: "REMOVED".to_string(),
                k8_version: "1.22".to_string(),
            };
            Ok(vec![m])
        }
    }
    let process = Te.process(Output::Table, "Filename").await;
    assert!(process.is_ok());
}

#[tokio::test]
async fn test_process_generate_csv() {
    struct Te;
    #[async_trait]
    impl Finder for Te {
        async fn find_deprecated_api(&self) -> anyhow::Result<Vec<TableDetails>> {
            let m = TableDetails {
                kind: "Kind".to_string(),
                namespace: "Test".to_string(),
                name: "Name".to_string(),
                supported_api_version: "REMOVED".to_string(),
                deprecated_api_version: "REMOVED".to_string(),
                k8_version: "1.22".to_string(),
            };
            Ok(vec![m])
        }
    }
    let process = Te.process(Output::Csv, "Filename").await;
    assert!(process.is_ok());
}
