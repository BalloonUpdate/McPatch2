use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::upload::UploadTarget;

const FILELIST: &str = ".filelist.txt";

/// 代表文件列表缓存功能。用来将 UploadTarget 的列目录操作转换为文件读取操作，
/// 也就是把文件列表存储在一个txt文件里，这样可以避免对真实 UploadTarget 执行列目录操作，以避免误删用户的其它文件
pub struct FileListCache<T> where T : UploadTarget {
    /// 真实 UploadTarget 对象
    target: T,

    /// 缓存的文件列表
    cache: Option<HashMap<String, u64>>,
}

impl<T> FileListCache<T> where T : UploadTarget {
    pub fn new(target: T) -> Self {
        Self {
            target,
            cache: None,
        }
    }

    /// 往文件列表里添加一个文件
    async fn add_file(&mut self, filename: &str, mtime: u64) {
        let filelist = self.cache.as_mut().unwrap();

        filelist.insert(filename.to_owned(), mtime);
    }

    /// 从文件列表里删除一个文件
    async fn remove_file(&mut self, filename: &str) {
        let filelist = self.cache.as_mut().unwrap();

        assert!(filelist.remove(filename).is_some());
    }

    /// 读取文件列表
    async fn read_filelist(&mut self) -> Result<Vec<(String, u64)>, String> {
        if self.cache.is_none() {
            let text = self.target.read(FILELIST).await?;

            let mut files = HashMap::new();

            if let Some(text) = text {
                let lines = text.split("\n")
                    .filter(|e| !e.is_empty());

                for line in lines {
                    let mut split = line.split(";");

                    let filename = split.next().unwrap().to_owned();
                    let mtime = split.next();

                    let mtime = match mtime {
                        Some(mt) => u64::from_str_radix(mt, 10).unwrap(),
                        None => {
                            println!("114514");

                            123456
                        },
                    };

                    files.insert(filename, mtime);
                }
            }

            self.cache = Some(files);
        }

        let result = self.cache.as_ref().unwrap().iter()
            .map(|e| (e.0.to_owned(), *e.1))
            .collect();

        Ok(result)
    }

    /// 写入文件列表
    async fn write_filelist(&mut self) -> Result<(), String> {
        assert!(self.cache.is_some());

        let content = self.cache.as_ref().unwrap()
            .iter()
            .map(|e| format!("{};{}", e.0, e.1))
            .collect::<Vec<_>>()
            .join("\n");

        self.target.write(FILELIST, &content).await
    }
}

impl<T> UploadTarget for FileListCache<T> where T : UploadTarget {
    async fn list(&mut self) -> Result<Vec<(String, u64)>, String> {
        // 这里列目录操作不会调用真实 UploadTatget 的方法，而是使用自己的文件缓存给替代掉
        let list = self.read_filelist().await?;

        Ok(list)
    }

    async fn read(&mut self, filename: &str) -> Result<Option<String>, String> {
        // 转发到真实 UploadTatget 上去处理
        self.target.read(filename).await
    }

    async fn write(&mut self, filename: &str, content: &str) -> Result<(), String> {
        // 转发到真实 UploadTatget 上去处理
        self.target.write(filename, content).await?;

        // 同时也要更新文件修改时间和文件列表
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        self.add_file(filename, now).await;

        // 主动更新文件列表
        self.write_filelist().await?;

        Ok(())
    }

    async fn upload(&mut self, filename: &str, filepath: PathBuf) -> Result<(), String> {
        let ts = filepath.metadata().unwrap().modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // 转发到真实 UploadTatget 上去处理
        self.target.upload(filename, filepath).await?;

        self.add_file(filename, ts).await;

        // 主动更新文件列表
        self.write_filelist().await?;

        Ok(())
    }

    async fn delete(&mut self, filename: &str) -> Result<(), String> {
        // 转发到真实 UploadTatget 上去处理
        self.target.delete(filename).await?;

        self.remove_file(filename).await;

        // 主动更新文件列表
        self.write_filelist().await?;

        Ok(())
    }
}