use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{Path, Query, Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{delete, get, head, patch, post, put},
};
use reggy_core::{
    blob::{
        close_chunked_session, get_unqiue_upload_location, read_blob_content, remove_blob,
        upload_chunk,
    },
    digest::Digest,
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
async fn get_blob(
    state: State<Arc<AppState>>,
    Path((name, digest)): Path<(String, String)>,
) -> impl IntoResponse {
    let blob = async || {
        let name = RepositoryName::new(&name, &state.hostname, Some(state.port))?;
        let digest = Digest::new(&digest)?;
        let (blob, headers) = read_blob_content(&name, &digest, &state.store).await?;
        return Ok::<_, RegistryError>((StatusCode::OK, create_headers(headers)?, blob));
    };

    match blob().await {
        Ok(result) => Ok(result),
        Err(RegistryError::BlobUnknown) => {
            Err((StatusCode::NOT_FOUND, "blob not found".to_string()))
        }
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.as_string())),
    }
}

async fn head_blobs(
    state: State<Arc<AppState>>,
    Path((name, digest)): Path<(String, String)>,
) -> impl IntoResponse {
    let exists = async || {
        let name = RepositoryName::new(&name, &state.hostname, Some(state.port))?;
        let digest = Digest::new(&digest)?;
        let (_, headers) = read_blob_content(&name, &digest, &state.store).await?;
        return Ok::<_, RegistryError>((StatusCode::OK, create_headers(headers)?));
    };

    match exists().await {
        Ok(result) => Ok(result),
        Err(RegistryError::BlobUnknown) => {
            Err((StatusCode::NOT_FOUND, "blob not found".to_string()))
        }
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.as_string())),
    }
}

async fn blob_delete(
    state: State<Arc<AppState>>,
    Path((name, digest)): Path<(String, String)>,
) -> impl IntoResponse {
    let delete = async || {
        let name = RepositoryName::new(&name, &state.hostname, Some(state.port))?;
        let digest = Digest::new(&digest)?;
        remove_blob(&name, &digest, &state.store).await?;
        return Ok::<_, RegistryError>(StatusCode::ACCEPTED);
    };

    match delete().await {
        Ok(result) => Ok(result),
        Err(RegistryError::BlobUnknown) => {
            Err((StatusCode::NOT_FOUND, "blob not found".to_string()))
        }
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.as_string())),
    }
}

async fn start_blob_upload_session(
    path: Path<String>,
    state: State<Arc<AppState>>,
) -> impl IntoResponse {
    let get_upload_headers = || {
        let name = RepositoryName::new(&path.0, &state.hostname, Some(state.port))?;
        let internal_headers = get_unqiue_upload_location(&name, true);
        let headers = create_headers(internal_headers)?;
        Ok::<_, RegistryError>((StatusCode::ACCEPTED, headers))
    };

    match get_upload_headers() {
        Ok(result) => Ok(result),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.as_string())),
    }
}

async fn blob_upload() {
    println!("blob_upload");
    todo!()
}

async fn blob_upload_patch(
    state: State<Arc<AppState>>,
    path: Path<(String, String)>,
    req: Request<Body>,
) -> impl IntoResponse {
    let headers = async || {
        let name = RepositoryName::new(&path.0.0, &state.hostname, Some(state.port))?;
        let chunk = to_bytes(req.into_body(), usize::MAX)
            .await
            .map_err(|e| RegistryError::Generic(e.to_string()))?
            .to_vec();
        let internal_headers = upload_chunk(&name, path.0.1, chunk, &state.store).await?;
        let headers = create_headers(internal_headers)?;
        Ok::<_, RegistryError>((StatusCode::ACCEPTED, headers))
    };

    match headers().await {
        Ok(result) => Ok(result),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.as_string())),
    }
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
            let headers = create_headers(internal_headers)?;
            return Ok::<_, RegistryError>((StatusCode::CREATED, headers));
        };

        return Err(RegistryError::Generic(
            "Reference must be a digest upon final upload.".to_string(),
        ));
    };

    match finalise().await {
        Ok(result) => Ok(result),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.as_string())),
    }
}

async fn mount_blob() {
    println!("mount_blob");
    todo!()
}

async fn download_blob() {
    println!("download_blob");
    todo!()
}

// Manifest
async fn get_manifests(
    state: State<Arc<AppState>>,
    Path((name, reference)): Path<(String, String)>,
) -> impl IntoResponse {
    let manifest = async || {
        let name = RepositoryName::new(&name, &state.hostname, Some(state.port))?;
        let reference = Reference::new(&reference)?;
        let (m, internal_headers) = pull_manifest(name, reference, &state.store).await?;
        let headers = create_headers(internal_headers)?;
        let body = serde_json::to_vec(&m).map_err(|e| RegistryError::Generic(e.to_string()))?;
        Ok::<_, RegistryError>((headers, body))
    };

    match manifest().await {
        Ok(result) => Ok(result),
        Err(RegistryError::ManifestUnknown) => {
            Err((StatusCode::NOT_FOUND, "Manifest not found".to_string()))
        }
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.as_string())),
    }
}

async fn head_manifests(
    state: State<Arc<AppState>>,
    Path((name, reference)): Path<(String, String)>,
) -> impl IntoResponse {
    let exists = async || {
        let name = RepositoryName::new(&name, &state.hostname, Some(state.port))?;
        let reference = Reference::new(&reference)?;
        let (_, internal_headers) = pull_manifest(name, reference, &state.store).await?;
        Ok::<_, RegistryError>(create_headers(internal_headers)?)
    };

    match exists().await {
        Ok(headers) => Ok(headers),
        Err(RegistryError::ManifestUnknown) => {
            Err((StatusCode::NOT_FOUND, "No manifest found.".to_string()))
        }
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
        Ok::<_, RegistryError>(create_headers(headers)?)
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

fn create_headers(headers: Headers) -> Result<HeaderMap, RegistryError> {
    let mut output = HeaderMap::new();
    for (k, v) in headers {
        let name = HeaderName::try_from(k).map_err(|e| RegistryError::Generic(e.to_string()))?;
        let value = HeaderValue::try_from(v).map_err(|e| RegistryError::Generic(e.to_string()))?;
        output.insert(name, value);
    }

    Ok(output)
}
