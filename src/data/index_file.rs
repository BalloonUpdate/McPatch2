use std::path::Path;

use crate::utility::join_string;

pub struct IndexFile {
    pub versions: Vec<String>
}

impl IndexFile {
    pub fn load(index_file: &Path) -> Self {
        let content = match std::fs::read_to_string(index_file) {
            Ok(content) => content,
            Err(_) => "".to_owned(),
        };
        let versions = content.split("\n")
            .map(|e| e.trim())
            .filter(|e| !e.is_empty())
            .map(|e| e.to_owned())
            .collect();

        Self { versions }
    }

    pub fn save(&self, index_file: &Path) {
        std::fs::write(index_file, join_string(self.versions.iter(), "\n")).unwrap()
    }

    pub fn contains_label(&self, label: &str) -> bool {
        self.versions.iter().any(|e| e == label)
    }
}