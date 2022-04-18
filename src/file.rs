use crate::utils::TableDetails;
use jwalk::{Parallelism, WalkDir};
use rayon::iter::ParallelBridge;
use rayon::prelude::ParallelIterator;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{channel, Sender};
use yaml_rust::YamlLoader;

pub(crate) fn search_files(t: Vec<Value>, loc: String) -> Vec<TableDetails> {
    let (sender, receiver) = channel();
    WalkDir::new(loc)
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
                            let docs = YamlLoader::load_from_str(&contents).unwrap();
                            // TODO: use combinators
                            for doc in docs {
                                if let Some(mut api_version) = doc["apiVersion"].as_str() {
                                    for z in t.iter() {
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

                                            let p = path
                                                .file_name()
                                                .unwrap()
                                                .to_str()
                                                .unwrap()
                                                .to_string();
                                            if z["removed"].as_str().unwrap().eq("true") {
                                                supported_api_version = "REMOVED".to_string();
                                                api_version = "REMOVED";

                                                if let Err(e) = sed.send((
                                                    doc["kind"].as_str().unwrap().to_string(),
                                                    supported_api_version,
                                                    api_version.to_string(),
                                                    doc["metadata"]["name"]
                                                        .as_str()
                                                        .unwrap()
                                                        .to_string(),
                                                    p,
                                                )) {
                                                    return Err(e);
                                                }
                                            } else if supported_api_version.ne(api_version) {
                                                if let Err(e) = sed.send((
                                                    doc["kind"].as_str().unwrap().to_string(),
                                                    supported_api_version,
                                                    api_version.to_string(),
                                                    doc["metadata"]["name"]
                                                        .as_str()
                                                        .unwrap()
                                                        .to_string(),
                                                    p,
                                                )) {
                                                    return Err(e);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            },
        )
        .expect("expected no send errors");
    let res: Vec<_> = receiver.iter().collect();
    let mut temp_table: Vec<TableDetails> = vec![];
    for (kind, supported_api_version, deprecated_api_version, name, path) in res {
        let p = path;
        let t = TableDetails {
            kind,
            namespace: p.to_owned(),
            name,
            supported_api_version,
            deprecated_api_version,
        };
        temp_table.push(t);
    }
    temp_table
}
