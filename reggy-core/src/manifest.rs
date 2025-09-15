use crate::{
    Response, digest::Digest, headers::Headers, reference::Reference,
    registry_error::RegistryError, repository_name::RepositoryName,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, future::Future};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: u32,
    pub media_type: String,
    pub config: Descriptor,
    #[serde(default)]
    pub layers: Vec<Descriptor>,
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Descriptor {
    pub media_type: String,
    pub digest: String,
    pub size: Option<u64>,
    #[serde(default)]
    pub urls: Vec<String>,
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

pub trait ManifestStore {
    fn read(
        &self,
        name: &RepositoryName,
        reference: &Reference,
    ) -> impl Future<Output = Result<Option<Manifest>, RegistryError>>;

    fn write(
        &self,
        name: &RepositoryName,
        reference: &Reference,
        manifest: &Manifest,
    ) -> impl Future<Output = Result<(), RegistryError>>;
}

pub async fn pull_manifest(
    name: RepositoryName,
    reference: Reference,
    manifest_store: &impl ManifestStore,
) -> Result<Response<Manifest>, RegistryError> {
    if let Some(manifest) = manifest_store.read(&name, &reference).await? {
        let digest = Digest::new(&manifest.config.digest).map_err(RegistryError::Generic)?;
        let mut headers = Headers::new(2);
        headers.insert_docker_content_digest(&digest);
        headers.insert_content_type(&manifest.media_type);
        return Ok((manifest, headers));
    } else {
        return Err(RegistryError::ManifestUnknown);
    }
}

pub async fn push_manifest(
    name: &RepositoryName,
    reference: &Reference,
    manifest: Manifest,
    manifest_store: &impl ManifestStore,
) -> Result<Headers, RegistryError> {
    let digest = Digest::new(&manifest.config.digest).map_err(RegistryError::Generic)?;
    if let Reference::Tag(_) = reference {
        manifest_store
            .write(name, &Reference::Digest(digest.clone()), &manifest)
            .await?;
    }
    manifest_store.write(name, reference, &manifest).await?;

    let mut headers = Headers::new(2);
    headers.insert_location(format!(
        "/v2/{}/manifests/{}",
        name.raw(),
        reference.into_string()
    ));
    headers.insert_docker_content_digest(&digest);
    Ok(headers)
}

pub async fn remove_manifest(
    name: &RepositoryName,
    reference: &Reference,
    manifest_store: &impl ManifestStore,
) -> Result<(), RegistryError> {
    todo!()
}
