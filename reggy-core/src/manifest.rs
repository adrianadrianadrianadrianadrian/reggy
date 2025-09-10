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

// TODO
pub async fn pull_manifest(
    name: RepositoryName,
    reference: Reference,
    supported_content_types: Vec<String>,
    manifest_store: &impl ManifestStore,
) -> Result<Option<Response<Manifest>>, RegistryError> {
    if let Some(manifest) = manifest_store.read(&name, &reference).await? {
        // if !supported_content_types.contains(&manifest.content_type) {
        //     return Err(RegistryError::Unsupported);
        // }

        let mut headers = Headers::new(1);
        if let Reference::Digest(digest) = reference {
            headers.insert_docker_content_digest(&digest);
        }

        return Ok(Some((manifest, headers)));
    }

    return Ok(None);
}

// TODO
pub async fn push_manifest(
    name: &RepositoryName,
    reference: &Reference,
    manifest: Manifest,
    manifest_store: &impl ManifestStore,
) -> Result<Headers, RegistryError> {
    manifest_store.write(name, reference, &manifest).await?;
    let mut headers = Headers::new(2);
    headers.insert_location(format!(
        "/v2/{}/manifests/{}",
        name.raw(),
        reference.into_string()
    ));
    let digest = Digest::new(&manifest.config.digest).unwrap();
    headers.insert_docker_content_digest(&digest);
    Ok(headers)
}
