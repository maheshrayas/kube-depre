use async_trait::async_trait;
use kube_depre::utils::*;

#[tokio::test]
async fn test_api() {
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
