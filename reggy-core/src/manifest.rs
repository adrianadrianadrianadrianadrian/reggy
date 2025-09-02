use crate::{
    headers::Headers, reference::Reference, registry_error::RegistryError,
    repository_name::RepositoryName,
};
use std::future::Future;

pub struct Manifest {
    pub content_type: String,
    pub media_type: Option<String>,
}

pub trait ManifestStore {
    fn read(
        &self,
        name: &RepositoryName,
        reference: &Reference,
    ) -> impl Future<Output = Result<Option<Manifest>, RegistryError>>;

    fn exists(
        &self,
        name: &RepositoryName,
        reference: &Reference,
    ) -> impl Future<Output = Result<bool, RegistryError>>;

    fn write(&self, manifest: Manifest) -> impl Future<Output = Result<(), RegistryError>>;
}

pub async fn read_manifest(
    name: RepositoryName,
    reference: Reference,
    supported_content_types: Vec<String>,
    manifest_store: &impl ManifestStore,
) -> Result<Option<(Manifest, Headers)>, RegistryError> {
    if let Some(manifest) = manifest_store.read(&name, &reference).await? {
        if !supported_content_types.contains(&manifest.content_type) {
            log::debug!("Manifest for repository: {name:?} and reference: {reference:?} is not supported by the client.");
            return Err(RegistryError::Unsupported);
        }

        let mut headers = Headers::new();
        if let Reference::Digest(digest) = reference {
            headers.insert_docker_content_digest(&digest);
        }

        return Ok(Some((manifest, headers)));
    }

    log::debug!("No manifest found for repository: {name:?} and reference: {reference:?}.");
    return Ok(None);
}
