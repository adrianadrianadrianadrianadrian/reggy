use axum::{
    body::Body,
    extract::{Path, Query},
    http::StatusCode,
    response::Response,
    routing::{delete, get, head, patch, post, put},
    Router,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct BlobUploadQuery {
    pub digest: Option<String>,
    pub mount: Option<String>,
    pub from: Option<String>,
}

// this is just temp for now
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/v2", get(async || StatusCode::OK))
        .route(
            "/v2/{name}/blobs/{digest}",
            get(async move |Path((name, digest)): Path<(String, String)>| {
                match get_blob(&name, &digest).await {
                    Some(b) => Response::builder()
                        .status(StatusCode::OK)
                        .header("Docker-Content-Digest", digest)
                        .body(Body::from(b))
                        .unwrap(),
                    None => Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap(),
                }
            })
            .head(head_blobs)
            .delete(blob_delete),
        )
        .route(
            "/v2/{name}/manifests/{reference}",
            get(get_manifests)
                .head(head_manifests)
                .put(manifest_put)
                .delete(manifest_delete),
        )
        .route(
            "/v2/{name}/blobs/uploads/",
            post(async |Query(query): Query<BlobUploadQuery>| {
                println!("{:?}", query);
                mount_blob().await
            }),
        )
        .route(
            "/v2/{name}/blobs/uploads/{reference}",
            patch(blob_upload_patch)
                .put(finalise_blob_upload)
                .get(download_blob),
        )
        .route("/v2/{name}/tags/list", get(get_tags)) // ?n={integer}&last={tagname}
        .route("/v2/{name}/referrers/{digest}", get(get_referrers)); //?artifactType={artifactType}",

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_blob(repository_name: &str, digest: &str) -> Option<Vec<u8>> {
    None
}
async fn head_blobs() {}
async fn get_manifests() {}
async fn head_manifests() {}
async fn blob_upload() {}
async fn blob_upload_patch() {}
async fn finalise_blob_upload() {}
async fn manifest_put() {}
async fn get_tags() {}
async fn manifest_delete() {}
async fn blob_delete() {}
async fn mount_blob() {}
async fn get_referrers() {}
async fn download_blob() {}
