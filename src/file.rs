use crate::utils::{Finder, TableDetails};
use async_trait::async_trait;
use jwalk::{Parallelism, WalkDir};
use rayon::iter::ParallelBridge;
use rayon::prelude::ParallelIterator;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use yaml_rust::{Yaml, YamlLoader};


type SenderChannel = Sender<(String, String, String, String, String)>;

#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct FileSystem {
    version: String,
    file_dir: String,
    deprecated_apis: Vec<Value>,
}

impl<'a> FileSystem {
    pub(crate) async fn new(file_dir: String, version: String) -> anyhow::Result<FileSystem> {
        Ok(FileSystem {
            file_dir,
            version: version.to_owned(),
            deprecated_apis: Self::get_deprecated_api(&version).await?,
        })
    }
    fn found_deprecated_api(
        &self,
        doc: Yaml,
        path: &Path,
        sed: &mut SenderChannel,
    ) -> anyhow::Result<()> {
        if let Some(mut api_version) = doc["apiVersion"].as_str() {
            for z in self.deprecated_apis.iter() {
                if z["kind"]
                    .as_str()
                    .unwrap()
                    .eq(doc["kind"].as_str().unwrap())
                {
                    let mut supported_api_version = format!(
                        "{}/{}",
                        z["group"].as_str().unwrap(),
                        z["version"].as_str().unwrap()
                    );

                    let p = path.file_name().unwrap().to_str().unwrap().to_string();
                    let mut send = false;
                    if z["removed"].as_str().unwrap().eq("true") {
                        supported_api_version = "REMOVED".to_string();
                        api_version = "REMOVED";
                        send = true
                    }
                    if supported_api_version.ne(api_version) || send.eq(&true) {
                        sed.send((
                            doc["kind"].as_str().unwrap().to_string(),
                            supported_api_version,
                            api_version.to_string(),
                            doc["metadata"]["name"].as_str().unwrap().to_string(),
                            p,
                        ))?
                    }
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl<'a> Finder for FileSystem {
    async fn find_deprecated_api(&self) -> anyhow::Result<Vec<TableDetails>> {
        let (sender, receiver) = channel();
        let _: anyhow::Result<()> = WalkDir::new(&self.file_dir)
            .parallelism(Parallelism::RayonNewPool(0))
            .into_iter()
            .par_bridge()
            .try_for_each_with(
                sender,
                |sed: &mut Sender<(String, String, String, String, String)>, op| {
                    let dir_entry = op.ok().unwrap();
                    if dir_entry.file_type().is_file() {
                        let path = dir_entry.path();
                        if let Some(yaml_file) = path.extension() {
                            if yaml_file.eq("yaml") {
                                let mut file = File::open(&path).expect("Unable to open file");
                                let mut contents = String::new();
                                file.read_to_string(&mut contents)
                                    .expect("Unable to read file");
                                let docs = YamlLoader::load_from_str(&contents)?;
                                for doc in docs {
                                    Self::found_deprecated_api(self, doc, &path, sed)?;
                                }
                            }
                        }
                    }
                    Ok(())
                },
            );
        let res: Vec<_> = receiver.iter().collect();
        let mut temp_table: Vec<TableDetails> = vec![];
        for (kind, supported_api_version, deprecated_api_version, name, path) in res {
            temp_table.push(TableDetails {
                kind,
                namespace: path,
                name,
                supported_api_version,
                deprecated_api_version,
            });
        }
        Ok(temp_table)
    }
}
