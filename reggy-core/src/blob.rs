use crate::{
    Response, digest::Digest, headers::Headers, registry_error::RegistryError,
    repository_name::RepositoryName,
};
use serde::{Deserialize, Serialize};
use std::future::Future;

#[derive(Serialize, Deserialize)]
pub struct Blob {
    pub metadata: BlobMetadata,
    pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct BlobMetadata {
    pub digest: Digest,
    pub content_length: usize,
}

pub trait BlobStore {
    fn read(
        &self,
        name: &RepositoryName,
        digest: &Digest,
    ) -> impl Future<Output = Result<Option<Blob>, RegistryError>>;

    fn write(
        &self,
        name: &RepositoryName,
        blob: &Blob,
    ) -> impl Future<Output = Result<(), RegistryError>>;

    fn write_chunk(
        &self,
        name: &RepositoryName,
        content: &Vec<u8>,
        session_id: &str,
    ) -> impl Future<Output = Result<(), RegistryError>>;

    fn read_chunk(
        &self,
        name: &RepositoryName,
        session_id: &str,
    ) -> impl Future<Output = Result<Option<Vec<u8>>, RegistryError>>;

    fn remove(
        &self,
        name: &RepositoryName,
        digest: &Digest,
    ) -> impl Future<Output = Result<(), RegistryError>>;
}

pub async fn read_blob_content(
    name: &RepositoryName,
    digest: &Digest,
    blob_store: &impl BlobStore,
) -> Result<Response<Vec<u8>>, RegistryError> {
    if let Some(blob) = blob_store.read(&name, &digest).await?.map(|b| b.content) {
        let mut headers = Headers::new(1);
        headers.insert_docker_content_digest(&digest);
        return Ok((blob, headers));
    }

    Err(RegistryError::BlobUnknown)
}

pub async fn read_metadata(
    name: RepositoryName,
    digest: Digest,
    blob_reader: &impl BlobStore,
) -> Result<Option<BlobMetadata>, RegistryError> {
    Ok(blob_reader.read(&name, &digest).await?.map(|b| b.metadata))
}

pub async fn blob_exists(
    name: RepositoryName,
    digest: Digest,
    blob_store: &impl BlobStore,
) -> Result<Response<bool>, RegistryError> {
    let mut headers = Headers::new(2);
    if let Some(Blob { metadata, .. }) = blob_store.read(&name, &digest).await? {
        headers.insert_docker_content_digest(&metadata.digest);
        headers.insert_content_length(metadata.content_length);
        return Ok((true, headers));
    }

    return Ok((false, headers));
}

pub async fn monolithic_upload(
    name: &RepositoryName,
    digest: Digest,
    blob_length: usize,
    blob_content: Vec<u8>,
    blob_store: &impl BlobStore,
) -> Result<Headers, RegistryError> {
    let content = blob_content.to_vec();
    if content.len() != blob_length {
        return Err(RegistryError::BlobUploadInvalid(format!(
            "Blob content length mismatch. Blob content length = {}. Provided length = {}.",
            content.len(),
            blob_length
        )));
    }

    if !digest.validate(&blob_content) {
        return Err(RegistryError::BlobUploadInvalid(format!(
            "Blob digest mismatch."
        )));
    }

    let blob = Blob {
        metadata: BlobMetadata {
            digest,
            content_length: blob_length,
        },
        content: blob_content,
    };

    blob_store.write(&name, &blob).await?;
    let mut headers = Headers::new(1);
    headers.insert_location(format!(
        "/v2/{}/blobs/{}",
        name.raw(),
        blob.metadata.digest.to_string()
    ));
    Ok(headers)
}

pub fn get_unqiue_upload_location(name: &RepositoryName, chunked_upload: bool) -> Headers {
    let session_id = uuid::Uuid::new_v4().to_string();
    let mut headers = Headers::new(3);
    headers.insert_location(format!("/v2/{}/blobs/uploads/{}", name.raw(), session_id));
    headers.insert_docker_upload_uuid(&session_id);
    if chunked_upload {
        headers.insert_content_length(0);
    }
    headers
}

pub async fn upload_chunk(
    name: &RepositoryName,
    session_id: String,
    // TODO: check start is end of current content
    //_content_range: Range,
    blob_content: Vec<u8>,
    blob_store: &impl BlobStore,
) -> Result<Headers, RegistryError> {
    let mut content = blob_store
        .read_chunk(name, &session_id)
        .await?
        .unwrap_or(vec![]);

    content.extend_from_slice(&blob_content);
    blob_store.write_chunk(name, &content, &session_id).await?;
    let mut headers = Headers::new(2);
    headers.insert_location(format!("/v2/{}/blobs/uploads/{}", name.raw(), session_id));
    if content.len() > 0 {
        headers.insert_range(0, content.len() - 1);
    }
    Ok(headers)
}

pub async fn close_chunked_session(
    name: &RepositoryName,
    digest: Digest,
    session_id: String,
    blob_content: Option<Vec<u8>>,
    blob_store: &impl BlobStore,
) -> Result<Headers, RegistryError> {
    // Q: What happens if we try close a session but the chunks thus far are empty?
    // Going to just unwrap to [] for now.
    let mut content = blob_store
        .read_chunk(name, &session_id)
        .await?
        .unwrap_or(vec![]);
    if let Some(final_layer) = blob_content {
        content.extend_from_slice(&final_layer);
    }

    if !digest.validate(&content) {
        // TODO: ?
    }

    let final_blob = &Blob {
        metadata: BlobMetadata {
            digest: digest.clone(),
            content_length: content.len(),
        },
        content,
    };

    blob_store.write(name, &final_blob).await?;
    let mut headers = Headers::new(1);
    headers.insert_location(format!("/v2/{}/blobs/{}", name.raw(), digest.to_string()));
    Ok(headers)
}

pub async fn remove_blob(
    name: &RepositoryName,
    digest: &Digest,
    blob_store: &impl BlobStore,
) -> Result<(), RegistryError> {
    blob_store.remove(name, digest).await
}
