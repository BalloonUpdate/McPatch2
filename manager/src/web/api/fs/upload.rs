use axum::body::Bytes;
use axum::extract::Multipart;
use axum::extract::State;
use axum::response::Response;
use tokio::io::AsyncWriteExt;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

pub async fn api_upload(State(state): State<WebState>, mut multipart: Multipart) -> Response {
    let mut path = Option::<String>::None;
    let mut data = Option::<Bytes>::None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name: &str = &field.name().unwrap().to_string();
        
        match name {
            "path" => {
                let text = field.text().await.unwrap();
                path = Some(text);
            },
            "file" =>  {
                let bytes = field.bytes().await.unwrap();
                data = Some(bytes);
            }
            _ => (),
        }
    }
    
    if path.is_none() {
        return PublicResponseBody::<()>::err("the filed 'path' is missing.");
    }

    if data.is_none() {
        return PublicResponseBody::<()>::err("the filed 'file' is missing.");
    }

    let path = path.unwrap();
    let data = data.unwrap();

    // 路径不能为空
    if path.is_empty() {
        return PublicResponseBody::<()>::err("parameter 'path' is empty, and it is not allowed.");
    }

    let file = state.config.workspace_dir.join(path);

    println!("upload: {:?}", file);

    if file.is_dir() {
        return PublicResponseBody::<()>::err("file is not writable.");
    }
    
    let mut f = tokio::fs::File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file)
        .await
        .unwrap();

    f.write_all(&data).await.unwrap();    

    PublicResponseBody::<()>::ok_no_data()
}