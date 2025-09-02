use crate::{
    digest::Digest, headers::Headers, range::Range, registry_error::RegistryError,
    repository_name::RepositoryName,
};
use std::future::Future;

pub struct Blob {
    pub metadata: BlobMetadata,
    pub content: Vec<u8>,
}

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

    fn read_metadata(
        &self,
        name: &RepositoryName,
        digest: &Digest,
    ) -> impl Future<Output = Result<Option<BlobMetadata>, RegistryError>>;
}

pub async fn read_blob_content(
    name: RepositoryName,
    digest: Digest,
    blob_store: &impl BlobStore,
) -> Result<Option<(Vec<u8>, Headers)>, RegistryError> {
    if let Some(blob) = blob_store.read(&name, &digest).await?.map(|b| b.content) {
        let mut headers = Headers::new();
        headers.insert_docker_content_digest(&digest);
        return Ok(Some((blob, headers)));
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
) -> Result<(bool, Option<Headers>), RegistryError> {
    if let Some(metadata) = blob_store.read_metadata(&name, &digest).await? {
        let mut headers = Headers::new();
        headers.insert_docker_content_digest(&metadata.digest);
        headers.insert_content_length(metadata.content_length);
        return Ok((true, Some(headers)));
    }

    return Ok((false, None));
}

pub async fn monolithic_upload(
    name: &RepositoryName,
    digest: Digest,
    blob_length: usize,
    blob_content: &[u8],
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

    let blob = Blob {
        metadata: BlobMetadata {
            digest,
            content_length: blob_length,
        },
        content: blob_content.to_vec(),
    };

    blob_store.write(&name, &blob).await?;
    let mut headers = Headers::new();
    headers.insert_location(format!(
        "/v2/{}/blobs/{}",
        name.raw(),
        blob.metadata.digest.hex()
    ));
    Ok(headers)
}

pub async fn get_unqiue_upload_location(name: &RepositoryName) -> Headers {
    let session_id = uuid::Uuid::new_v4().to_string();
    let mut headers = Headers::new();
    headers.insert_location(format!("/v2/{}/blobs/upload/{}", name.raw(), session_id));
    headers.insert_docker_upload_uuid(&session_id);
    headers.insert_content_length(0);
    headers
}

pub async fn upload_chunk(
    name: &RepositoryName,
    session_id: String,
    content_range: Range,
    blob_length: usize,
    blob_content: &[u8],
    blob_store: &impl BlobStore,
) -> Result<Headers, RegistryError> {
    todo!()
}
