use std::collections::HashSet;
use std::path::PathBuf;

use crate::upload::SyncTarget;

const FILELIST: &str = ".filelist.txt";

pub struct FileListCache<T> where T : SyncTarget {
    target: T,
    cache: Option<HashSet<String>>,
}

impl<T> FileListCache<T> where T : SyncTarget {
    pub fn new(target: T) -> Self {
        Self {
            target,
            cache: None,
        }
    }

    async fn add_file(&mut self, filename: &str) {
        let filelist = self.cache.as_mut().unwrap();

        filelist.insert(filename.to_owned());
    }

    async fn remove_file(&mut self, filename: &str) {
        let filelist = self.cache.as_mut().unwrap();

        filelist.remove(filename);
    }

    async fn read_filelist(&mut self) -> Result<Vec<String>, String> {
        if self.cache.is_none() {
            let text = self.target.read(FILELIST).await?;

            let files = match text {
                Some(text) => text
                    .split("\n")
                    .map(|e| e.trim())
                    .filter(|e| !e.is_empty())
                    .map(|e| e.to_owned())
                    .collect(),
                None => HashSet::new(),
            };

            self.cache = Some(files);
        }

        Ok(self.cache.as_ref().unwrap().iter().map(|e| e.to_owned()).collect())
    }

    async fn write_filelist(&mut self) -> Result<(), String> {
        assert!(self.cache.is_some());

        let content = self.cache.as_ref().unwrap()
            .iter()
            .map(|e| e.to_owned())
            .collect::<Vec<_>>()
            .join("\n");

        self.target.write(FILELIST, &content).await
    }
}

impl<T> SyncTarget for FileListCache<T> where T : SyncTarget {
    async fn list(&mut self) -> Result<Vec<String>, String> {
        let list = self.read_filelist().await?;

        Ok(list)
    }

    async fn read(&mut self, filename: &str) -> Result<Option<String>, String> {
        self.target.read(filename).await
    }

    async fn write(&mut self, filename: &str, content: &str) -> Result<(), String> {
        self.target.write(filename, content).await?;

        self.add_file(filename).await;
        self.write_filelist().await?;

        Ok(())
    }

    async fn upload(&mut self, filename: &str, filepath: PathBuf) -> Result<(), String> {
        self.target.upload(filename, filepath).await?;

        self.add_file(filename).await;
        self.write_filelist().await?;

        Ok(())
    }

    async fn delete(&mut self, filename: &str) -> Result<(), String> {
        self.target.delete(filename).await?;

        self.remove_file(filename).await;
        self.write_filelist().await?;

        Ok(())
    }
}