use async_trait::async_trait;
use kube_depre::{FileSystem, Finder, TableDetails};
use std::path::PathBuf;

#[tokio::test]
async fn test_find_deprecated_api_in_files() {
    struct Te;
    #[async_trait]
    impl Finder for Te {
        async fn find_deprecated_api(&self) -> anyhow::Result<Vec<TableDetails>> {
            Ok(vec![])
        }
    }
    let versions: Vec<&str> = vec!["1.25"];
    let mut absolute_path = std::env::current_dir().unwrap();
    let relative_path = PathBuf::from("tests/data");
    absolute_path.push(relative_path);
    let new_file = FileSystem::new(
        absolute_path.into_os_string().into_string().unwrap(),
        versions,
    )
    .await;
    let m = new_file.unwrap().find_deprecated_api().await.unwrap();
    assert_eq!(m.len(), 2);
}

#[tokio::test]
async fn test_find_deprecated_api_in_files_multiple_version() {
    struct Te;
    #[async_trait]
    impl Finder for Te {
        async fn find_deprecated_api(&self) -> anyhow::Result<Vec<TableDetails>> {
            Ok(vec![])
        }
    }
    let versions: Vec<&str> = vec!["1.25", "1.22"];
    let mut absolute_path = std::env::current_dir().unwrap();
    let relative_path = PathBuf::from("tests/data");
    absolute_path.push(relative_path);
    let new_file = FileSystem::new(
        absolute_path.into_os_string().into_string().unwrap(),
        versions,
    )
    .await;
    let m = new_file.unwrap().find_deprecated_api().await.unwrap();
    assert_eq!(m.len(), 2);
}

#[tokio::test]
async fn test_find_deprecated_api_in_files_with_no_matching_version() {
    struct Te;
    #[async_trait]
    impl Finder for Te {
        async fn find_deprecated_api(&self) -> anyhow::Result<Vec<TableDetails>> {
            Ok(vec![])
        }
    }
    let versions: Vec<&str> = vec!["1.26"];
    let mut absolute_path = std::env::current_dir().unwrap();
    let relative_path = PathBuf::from("tests/data");
    absolute_path.push(relative_path);
    let new_file = FileSystem::new(
        absolute_path.into_os_string().into_string().unwrap(),
        versions,
    )
    .await;
    let m = new_file.unwrap().find_deprecated_api().await.unwrap();
    assert_eq!(m.len(), 0);
}
