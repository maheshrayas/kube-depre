use async_trait::async_trait;
use kube_depre::{Cluster, Finder, TableDetails};

#[tokio::test]
async fn test_find_deprecated_api_in_cluster() {
    struct Te;
    #[async_trait]
    impl Finder for Te {
        async fn find_deprecated_api(&self) -> anyhow::Result<Vec<TableDetails>> {
            Ok(vec![])
        }
    }
    let versions: Vec<&str> = vec!["1.25"];
    let c = Cluster::new(versions).await;
    let m = c.unwrap().find_deprecated_api().await.unwrap();
    assert_eq!(m.len(), 2);
}
