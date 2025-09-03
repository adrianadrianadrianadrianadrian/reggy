use axum::{
    body::Body,
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, head, patch, post, put},
    Router,
};
use reggy_core::{manifest::pull_manifest, reference::Reference, repository_name::RepositoryName};
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
                get_blob(&name, &digest).await;
            })
            .head(head_blobs)
            .delete(blob_delete),
        )
        .route(
            "/v2/{name}/manifests/{reference}",
            get(
                async move |Path((name, reference)): Path<(String, String)>| {
                    get_manifests(&name, &reference).await
                },
            )
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

async fn get_blob(repository_name: &str, digest: &str) {}
async fn head_blobs() {}
async fn get_manifests(repository_name: &str, reference: &str) -> impl IntoResponse {
    let name = RepositoryName::new(repository_name, "", None);
    let reference = Reference::new(reference);
    //let manifest = pull_manifest(name, reference, vec![], todo!()).await;

    return Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap();
}
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
