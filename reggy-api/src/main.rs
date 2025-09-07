use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, head, patch, post, put},
};
use reggy_core::{
    headers::Headers,
    manifest::{Manifest, ManifestStore, pull_manifest},
    point_read_store::PointReadStore,
    reference::Reference,
    registry_error::RegistryError,
    repository_name::RepositoryName,
};
use reggy_fs::FsPersistence;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct BlobUploadQuery {
    pub digest: Option<String>,
    pub mount: Option<String>,
    pub from: Option<String>,
}

#[derive(Clone)]
struct AppState<M: ManifestStore> {
    hostname: String,
    port: u16,
    manifest_store: M,
}

// this is just temp for now
#[tokio::main]
async fn main() {
    let fs = FsPersistence {
        root_dir: "./registry".to_string(),
    };
    let store = PointReadStore::new(fs);
    let state = std::sync::Arc::new(AppState {
        hostname: "localhost".to_string(),
        port: 3000,
        manifest_store: store,
    });

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
        .route("/v2/{name}/referrers/{digest}", get(get_referrers)) //?artifactType={artifactType}"
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", state.hostname, state.port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_blob(repository_name: &str, digest: &str) {}
async fn head_blobs() {}

async fn get_manifests<M: ManifestStore>(
    state: State<Arc<AppState<M>>>,
    Path((name, reference)): Path<(String, String)>,
) -> impl IntoResponse {
    let read_manifest = async || -> Result<_, RegistryError> {
        let name = RepositoryName::new(&name, &state.hostname, Some(state.port))?;
        let reference = Reference::new(&reference)?;
        Ok(pull_manifest(name, reference, vec![], &state.manifest_store).await?)
    };

    match read_manifest().await {
        Ok(_) => StatusCode::OK,
        Err(_) => todo!(),
    }
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
