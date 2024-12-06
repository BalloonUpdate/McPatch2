use axum::extract::State;
use axum::response::Response;
use serde::Serialize;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

#[derive(Serialize)]
pub struct ResponseData {
    pub dev: String,
    pub used: u64,
    pub total: u64,
}

pub async fn api_disk_info(State(state): State<WebState>) -> Response {
    #[allow(unused_mut)]
    let mut path = state.app_path.working_dir.canonicalize().unwrap().to_str().unwrap().to_string();

    #[cfg(target_os = "windows")]
    if path.starts_with(r"\\?\") {
        path = path[4..].to_owned();
    }

    let one_peta_bytes: u64 = 1 * 1024 * 1024 * 1024 * 1024 * 1024;
    let mut usages = (one_peta_bytes, one_peta_bytes, "none".to_owned());

    let disks = sysinfo::Disks::new_with_refreshed_list();

    for disk in disks.list() {
        let name = disk.name().to_str().unwrap().to_owned();
        let mount = disk.mount_point().to_str().unwrap().replace(r"\\", r"\");

        if path.starts_with(&mount) {
            let total = disk.total_space();
            let available = disk.available_space();

            usages = (total - available, total, name);
        }
    }

    PublicResponseBody::<ResponseData>::ok(ResponseData {
        used: usages.0,
        total: usages.1,
        dev: usages.2,
    })
}