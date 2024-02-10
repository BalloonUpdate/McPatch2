use std::collections::HashMap;
use std::collections::LinkedList;
use std::io::Seek;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;

use crate::data::version_meta::FileChange;
use crate::data::version_meta::VersionMeta;
use crate::diff::abstract_file::AbstractFile;
use crate::diff::diff::Diff;
use crate::diff::disk_file::DiskFile;
use crate::diff::history_file::HistoryFile;
use crate::utility::counted_write::CountedWrite;

pub struct VersionWriter {
    write: CountedWrite<std::fs::File>
}

impl VersionWriter {
    pub fn new(version: impl AsRef<Path>) -> Self {
        let open = std::fs::File::options().create(true).truncate(true).write(true).open(version).unwrap();
        let counter = CountedWrite::new(open);

        Self { write: counter }
    }

    pub fn write_diff(&mut self, diff: &Diff<'_, DiskFile, HistoryFile>, workspace: &Path) {
        let mut tar = tar::Builder::new(&mut self.write);
        let mut header = tar::Header::new_gnu();
        
        // 写入地址文件
        let placeholder = [b' '; 512];
        header.set_size(placeholder.len() as u64);
        tar.append_data(&mut header, "addr.txt", std::io::Cursor::new(&placeholder)).unwrap();

        // 写入每个更新的文件数据
        let mut addresses = HashMap::new();
        for f in &diff.updated_files {
            let path = f.path().to_owned();
            let disk_file = workspace.join(&path);
            
            // 记录当前指针位置
            addresses.insert(path.clone(), tar.get_ref().count());

            let open = std::fs::File::options().read(true).open(disk_file).unwrap();
            header.set_size(f.len());
            tar.append_data(&mut header, path, open).unwrap();
        }
        
        // 写入元数据
        let metadata_offset = tar.get_ref().count();
        let metadata = VersionMeta::new("no logs".to_owned(), diff_to_changes(diff, &addresses));
        let file_content = metadata.save();
        let file_content = file_content.as_bytes();
        header.set_size(file_content.len() as u64);
        tar.append_data(&mut header, "metadata.txt", std::io::Cursor::new(&file_content)).unwrap();

        // 写入完毕
        tar.into_inner().unwrap();
    
        // 直接更新地址文件的内容
        self.write.seek(std::io::SeekFrom::Start(512)).unwrap();
        self.write.write_all(format!("{:}", metadata_offset).as_bytes()).unwrap();
    }
}

fn diff_to_changes(diff: &Diff<'_, DiskFile, HistoryFile>, addresses: &HashMap<String, u64>) -> LinkedList<FileChange> {
    let mut changes = LinkedList::new();

    for f in &diff.deleted_files {
        changes.push_back(FileChange::DeleteFile { 
            path: f.path().to_owned() 
        })
    }

    for f in &diff.created_folders {
        changes.push_back(FileChange::CreateFolder { 
            path: f.path().to_owned() 
        })
    }

    for f in &diff.renamed_files {
        changes.push_back(FileChange::MoveFile {
            from: f.0.path().to_owned(), 
            to: f.1.path().to_owned()
        })
    }

    for f in &diff.updated_files {
        changes.push_back(FileChange::UpdateFile { 
            path: f.path().to_owned(), 
            hash: f.hash().to_owned(), 
            len: f.len(), 
            modified: f.modified(), 
            offset: *addresses.get(f.path().deref()).unwrap()
        })
    }

    for f in &diff.deleted_folders {
        changes.push_back(FileChange::DeleteFolder { 
            path: f.path().to_owned() 
        })
    }

    changes
}