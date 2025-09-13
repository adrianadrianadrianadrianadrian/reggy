use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{Path, Query, Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{delete, get, head, patch, post, put},
};
use reggy_core::{
    blob::{close_chunked_session, get_unqiue_upload_location, read_blob_content, upload_chunk},
    headers::Headers,
    manifest::{Manifest, pull_manifest, push_manifest, remove_manifest},
    reference::Reference,
    registry_error::RegistryError,
    repository_name::RepositoryName,
};
use reggy_fs::FsStore;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Debug)]
struct BlobUploadQuery {
    pub digest: String,
}

#[derive(Clone)]
struct AppState {
    hostname: String,
    port: u16,
    store: FsStore,
}

// this is all just temp for now
#[tokio::main]
async fn main() {
    let fs = FsStore {
        root_dir: "/home/adrian/code/reggy/registry".to_string(),
    };
    let state = std::sync::Arc::new(AppState {
        hostname: "localhost".to_string(),
        port: 3000,
        store: fs,
    });

    let app = Router::new()
        .route("/v2", get(async || StatusCode::OK))
        .route(
            "/v2/{name}/blobs/{digest}",
            get(get_blob).head(head_blobs).delete(blob_delete),
        )
        .route(
            "/v2/{name}/manifests/{reference}",
            get(get_manifests)
                .head(head_manifests)
                .put(put_manifest)
                .delete(delete_manifest),
        )
        .route("/v2/{name}/blobs/uploads/", post(start_blob_upload_session))
        .route(
            "/v2/{name}/blobs/uploads/{reference}",
            patch(blob_upload_patch)
                .put(finalise_blob_upload)
                .post(blob_upload)
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

// Blobs
async fn start_blob_upload_session(
    path: Path<String>,
    state: State<Arc<AppState>>,
) -> impl IntoResponse {
    let name = RepositoryName::new(&path.0, &state.hostname, Some(state.port)).unwrap();
    let internal_headers = get_unqiue_upload_location(&name, true);
    let headers = create_headers(internal_headers).unwrap();
    (StatusCode::ACCEPTED, headers)
}

async fn get_blob() {
    println!("get_blob");
}

async fn head_blobs(
    state: State<Arc<AppState>>,
    Path((name, reference)): Path<(String, String)>,
) -> impl IntoResponse {
    let name = RepositoryName::new(&name, &state.hostname, Some(state.port)).unwrap();
    if let Reference::Digest(digest) = Reference::new(&reference).unwrap() {
        println!("{:?}", digest);
        if let Ok((_, headers)) = read_blob_content(&name, &digest, &state.store).await {
            return (StatusCode::OK, create_headers(headers).unwrap());
        }
    }

    (StatusCode::NOT_FOUND, HeaderMap::new())
}

async fn blob_upload() {
    println!("blob_upload");
}

async fn blob_upload_patch(
    state: State<Arc<AppState>>,
    path: Path<(String, String)>,
    req: Request<Body>,
) -> impl IntoResponse {
    let name = RepositoryName::new(&path.0.0, &state.hostname, Some(state.port)).unwrap();
    let chunk = to_bytes(req.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    let internal_headers = upload_chunk(&name, path.0.1, chunk, &state.store)
        .await
        .unwrap();

    let headers = create_headers(internal_headers).unwrap();
    println!("blob_upload_patch");
    (StatusCode::ACCEPTED, headers)
}

async fn finalise_blob_upload(
    state: State<Arc<AppState>>,
    path: Path<(String, String)>,
    Query(query): Query<BlobUploadQuery>,
    req: Request<Body>,
) -> impl IntoResponse {
    println!("finalise_blob_upload");
    let finalise = async || {
        let name = RepositoryName::new(&path.0.0, &state.hostname, Some(state.port))?;
        let session_id = &path.0.1;
        let reference = Reference::new(&query.digest);

        if let Ok(Reference::Digest(digest)) = reference {
            let last_chunk = to_bytes(req.into_body(), usize::MAX)
                .await
                .map_err(|e| RegistryError::Generic(e.to_string()))?
                .to_vec();

            let mut last = None;
            if last_chunk.len() > 0 {
                last = Some(last_chunk);
            }

            let internal_headers =
                close_chunked_session(&name, digest, session_id.to_string(), last, &state.store)
                    .await?;
            return Ok::<_, RegistryError>(
                create_headers(internal_headers).map_err(RegistryError::Generic)?,
            );
        };

        return Err(RegistryError::Generic("asdfasd".to_string()));
    };

    match finalise().await {
        Ok(headers) => (StatusCode::CREATED, headers),
        Err(_) => todo!(),
    };
}

async fn blob_delete() {
    println!("blob_delete");
}

async fn mount_blob() {
    println!("mount_blob");
}

async fn download_blob() {
    println!("download_blob");
}

// Manifest
async fn get_manifests(
    state: State<Arc<AppState>>,
    Path((name, reference)): Path<(String, String)>,
) -> impl IntoResponse {
    let get = async || {
        let name = RepositoryName::new(&name, &state.hostname, Some(state.port))?;
        let reference = Reference::new(&reference)?;
        Ok::<_, RegistryError>(pull_manifest(name, reference, &state.store).await?)
    };

    match get().await {
        Ok(Some((m, h))) => match create_headers(h) {
            Ok(headers) => match serde_json::to_vec(&m) {
                Ok(manifest) => Ok((headers, manifest)),
                Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
            },
            Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err)),
        },
        Ok(None) => Err((StatusCode::NOT_FOUND, "Manifest not found".to_string())),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.as_string())),
    }
}

async fn head_manifests(
    state: State<Arc<AppState>>,
    Path((name, reference)): Path<(String, String)>,
) -> impl IntoResponse {
    let head = async || {
        let name = RepositoryName::new(&name, &state.hostname, Some(state.port))?;
        let reference = Reference::new(&reference)?;
        Ok::<_, RegistryError>(pull_manifest(name, reference, &state.store).await?)
    };

    match head().await {
        Ok(Some((_, h))) => match create_headers(h) {
            Ok(headers) => Ok(headers),
            Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err)),
        },
        Ok(None) => Err((StatusCode::NOT_FOUND, "No manifest found.".to_string())),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.as_string())),
    }
}

async fn put_manifest(
    state: State<Arc<AppState>>,
    Path((name, reference)): Path<(String, String)>,
    req: Request<Body>,
) -> impl IntoResponse {
    let put = async || {
        let name = RepositoryName::new(&name, &state.hostname, Some(state.port))?;
        let reference = Reference::new(&reference)?;
        let data = to_bytes(req.into_body(), usize::MAX)
            .await
            .map_err(|e| RegistryError::Generic(e.to_string()))?
            .to_vec();
        let manifest: Manifest =
            serde_json::from_slice(&data).map_err(|e| RegistryError::Generic(e.to_string()))?;
        let headers = push_manifest(&name, &reference, manifest, &state.store).await?;
        Ok::<_, RegistryError>(create_headers(headers).map_err(RegistryError::Generic)?)
    };

    match put().await {
        Ok(headers) => Ok((StatusCode::CREATED, headers)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.as_string())),
    }
}

async fn delete_manifest(
    state: State<Arc<AppState>>,
    Path((name, reference)): Path<(String, String)>,
) -> impl IntoResponse {
    let delete = async || {
        let name = RepositoryName::new(&name, &state.hostname, Some(state.port))?;
        let reference = Reference::new(&reference)?;
        Ok::<_, RegistryError>(remove_manifest(&name, &reference, &state.store).await?)
    };

    match delete().await {
        Ok(()) => Ok(StatusCode::ACCEPTED),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.as_string())),
    }
}

// Tags
async fn get_tags() {
    println!("get_tags");
}

async fn get_referrers() {
    println!("get_referrers");
}

fn create_headers(headers: Headers) -> Result<HeaderMap, String> {
    let mut output = HeaderMap::new();
    for (k, v) in headers {
        let name = HeaderName::try_from(k).map_err(|e| e.to_string())?;
        let value = HeaderValue::try_from(v).map_err(|e| e.to_string())?;
        output.insert(name, value);
    }

    Ok(output)
}
