use crate::{
    blob::{Blob, BlobStore},
    digest::Digest,
    manifest::{Manifest, ManifestStore},
    reference::Reference,
    registry_error::RegistryError,
    repository_name::RepositoryName,
};
use std::future::Future;

pub trait PointReadPersistence {
    fn read(&self, id: String) -> impl Future<Output = Result<Option<Vec<u8>>, String>>;
    fn write(&self, id: String, data: &Vec<u8>) -> impl Future<Output = Result<(), String>>;
}

#[derive(Clone)]
pub struct PointReadStore<P: PointReadPersistence> {
    persistence: P,
}

impl<P: PointReadPersistence> PointReadStore<P> {
    pub fn new(persistence: P) -> Self {
        Self { persistence }
    }
}

impl<P: PointReadPersistence> BlobStore for PointReadStore<P> {
    async fn read(
        &self,
        name: &RepositoryName,
        digest: &Digest,
    ) -> Result<Option<Blob>, RegistryError> {
        if let Some(data) = self
            .persistence
            .read(blob_id(name, digest))
            .await
            .map_err(RegistryError::Generic)?
        {
            return serde_json::from_slice(&data)
                .map_err(|e| RegistryError::Generic(e.to_string()));
        }

        Ok(None)
    }

    async fn write(&self, name: &RepositoryName, blob: &Blob) -> Result<(), RegistryError> {
        let data = serde_json::to_vec(blob).map_err(|e| RegistryError::Generic(e.to_string()))?;
        self.persistence
            .write(blob_id(name, &blob.metadata.digest), &data)
            .await
            .map_err(RegistryError::Generic)
    }

    async fn write_chunk(
        &self,
        name: &RepositoryName,
        content: &Vec<u8>,
        session_id: &str,
    ) -> Result<(), RegistryError> {
        self.persistence
            .write(blob_chunk_id(name, session_id), content)
            .await
            .map_err(RegistryError::Generic)
    }

    async fn read_chunk(
        &self,
        name: &RepositoryName,
        session_id: &str,
    ) -> Result<Option<Vec<u8>>, RegistryError> {
        self.persistence
            .read(blob_chunk_id(name, session_id))
            .await
            .map_err(RegistryError::Generic)
    }
}

impl<P: PointReadPersistence> ManifestStore for PointReadStore<P> {
    async fn read(
        &self,
        name: &RepositoryName,
        reference: &Reference,
    ) -> Result<Option<Manifest>, RegistryError> {
        if let Some(data) = self
            .persistence
            .read(manifest_id(name, reference))
            .await
            .map_err(RegistryError::Generic)?
        {
            return serde_json::from_slice(&data)
                .map_err(|e| RegistryError::Generic(e.to_string()));
        }

        Ok(None)
    }

    async fn write(
        &self,
        name: &RepositoryName,
        reference: &Reference,
        manifest: Manifest,
    ) -> Result<(), RegistryError> {
        let data =
            serde_json::to_vec(&manifest).map_err(|e| RegistryError::Generic(e.to_string()))?;

        self.persistence
            .write(manifest_id(name, reference), &data)
            .await
            .map_err(RegistryError::Generic)
    }
}

fn manifest_id(name: &RepositoryName, reference: &Reference) -> String {
    format!("{}/manifest/{}", name.raw(), reference.into_string())
}

fn blob_id(name: &RepositoryName, digest: &Digest) -> String {
    format!("{}/blob/{}", name.raw(), digest.hex())
}

fn blob_chunk_id(name: &RepositoryName, session_id: &str) -> String {
    format!("{}/blob_chunk/{}", name.raw(), session_id)
}
