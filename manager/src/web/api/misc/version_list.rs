use axum::extract::State;
use axum::response::Response;
use serde::Serialize;
use shared::data::index_file::IndexFile;
use shared::data::version_meta::FileChange;
use shared::data::version_meta::VersionMeta;

use crate::core::tar_reader::TarReader;
use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Serialize)]
pub struct ResponseBody {
    /// 要删除的文件路径
    versions: Vec<Version>,
}

#[derive(Serialize)]
pub struct Version {
    pub label: String,
    pub size: u64,
    pub change_logs: String,
}

pub async fn api_version_list(State(state): State<WebState>) -> Response {
    let index_file = IndexFile::load_from_file(&state.apppath.index_file);

    let mut metas = Vec::<VersionMeta>::new();

    for v in &index_file {
        let mut reader = TarReader::new(state.apppath.public_dir.join(&v.filename));
        let meta_group = reader.read_metadata_group(v.offset, v.len);

        for meta in meta_group {
            metas.push(meta);
        }
    }
    
    let mut versions = Vec::<Version>::new();

    for meta in metas {
        let mut total_size = 0u64;

        for change in &meta.changes {
            if let FileChange::UpdateFile { len, .. } = change {
                total_size += len;
            }
        }
        
        versions.push(Version {
            label: meta.label, 
            size: total_size, 
            change_logs: meta.logs,
        });
    }

    PublicResponseBody::<ResponseBody>::ok(ResponseBody { versions })
}