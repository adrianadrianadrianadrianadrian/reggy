use reggy_core::{
    blob::{Blob, BlobStore},
    digest::Digest,
    manifest::{Manifest, ManifestStore},
    reference::Reference,
    registry_error::RegistryError,
    repository_name::RepositoryName,
    tag::Tag,
};
use std::{fs, path::Path};

#[derive(Clone)]
pub struct FsStore {
    pub root_dir: String,
}

impl BlobStore for FsStore {
    async fn read(
        &self,
        name: &RepositoryName,
        digest: &Digest,
    ) -> Result<Option<Blob>, RegistryError> {
        let raw_path = path(&self.root_dir, &blob_id(name, digest));
        if let Some(data) = read_file(Path::new(&raw_path)).map_err(RegistryError::Generic)? {
            return serde_json::from_slice(&data)
                .map_err(|e| RegistryError::Generic(e.to_string()));
        }

        Ok(None)
    }

    async fn write(&self, name: &RepositoryName, blob: &Blob) -> Result<(), RegistryError> {
        let raw_path = path(&self.root_dir, &blob_id(name, &blob.metadata.digest));
        let data = serde_json::to_vec(blob).map_err(|e| RegistryError::Generic(e.to_string()))?;
        write_file(Path::new(&raw_path), &data).map_err(RegistryError::Generic)
    }

    async fn write_chunk(
        &self,
        name: &RepositoryName,
        content: &Vec<u8>,
        session_id: &str,
    ) -> Result<(), RegistryError> {
        let raw_path = path(&self.root_dir, &blob_chunk_id(name, session_id));
        write_file(Path::new(&raw_path), content).map_err(RegistryError::Generic)
    }

    async fn read_chunk(
        &self,
        name: &RepositoryName,
        session_id: &str,
    ) -> Result<Option<Vec<u8>>, RegistryError> {
        let raw_path = path(&self.root_dir, &blob_chunk_id(name, session_id));
        read_file(Path::new(&raw_path)).map_err(RegistryError::Generic)
    }

    async fn remove(&self, name: &RepositoryName, digest: &Digest) -> Result<(), RegistryError> {
        let raw_path = path(&self.root_dir, &blob_id(name, &digest));
        std::fs::remove_file(Path::new(&raw_path))
            .map_err(|e| RegistryError::Generic(e.to_string()))
    }
}

impl ManifestStore for FsStore {
    async fn read(
        &self,
        name: &RepositoryName,
        reference: &Reference,
    ) -> Result<Option<Manifest>, RegistryError> {
        let raw_path = path(&self.root_dir, &manifest_id(name, reference));
        if let Some(data) = read_file(Path::new(&raw_path)).map_err(RegistryError::Generic)? {
            return serde_json::from_slice(&data)
                .map_err(|e| RegistryError::Generic(e.to_string()));
        }

        Ok(None)
    }

    async fn write(
        &self,
        name: &RepositoryName,
        reference: &Reference,
        manifest: &Manifest,
    ) -> Result<(), RegistryError> {
        let data =
            serde_json::to_vec(&manifest).map_err(|e| RegistryError::Generic(e.to_string()))?;
        let raw_manifest_path = path(&self.root_dir, &manifest_id(name, reference));
        write_file(Path::new(&raw_manifest_path), &data).map_err(RegistryError::Generic)?;

        if let Reference::Tag(t) = reference {
            let raw_tag_path = path(&self.root_dir, &tags_id(name));
            let raw_tags = read_file(Path::new(&raw_tag_path))
                .map_err(|e| RegistryError::Generic(e.to_string()))?
                .unwrap_or_default();
            let mut current_tags: Vec<String> =
                serde_json::from_slice(&raw_tags).unwrap_or_default();
            let raw_tag = t.raw();
            if !current_tags.contains(&raw_tag) {
                current_tags.push(raw_tag);
                let updated = serde_json::to_vec(&current_tags)
                    .map_err(|e| RegistryError::Generic(e.to_string()))?;
                write_file(Path::new(&raw_tag_path), &updated)
                    .map_err(|e| RegistryError::Generic(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn read_tags(&self, name: &RepositoryName) -> Result<Vec<Tag>, RegistryError> {
        let raw_tag_path = path(&self.root_dir, &tags_id(name));
        let raw_tags = read_file(Path::new(&raw_tag_path))
            .map_err(|e| RegistryError::Generic(e.to_string()))?
            .unwrap_or_default();
        let mut output = vec![];
        let data: Vec<String> =
            serde_json::from_slice(&raw_tags).map_err(|e| RegistryError::Generic(e.to_string()))?;
        for raw_tag in data {
            output.push(Tag::new(&raw_tag)?);
        }
        Ok(output)
    }
}

fn path(root_dir: &str, id: &str) -> String {
    format!("{}/{}", root_dir, id)
}

fn manifest_id(name: &RepositoryName, reference: &Reference) -> String {
    format!("{}/manifest/{}", name.raw(), reference.into_string())
}

fn tags_id(name: &RepositoryName) -> String {
    format!("{}/tags", name.raw())
}

fn blob_id(name: &RepositoryName, digest: &Digest) -> String {
    format!("{}/blob/{}", name.raw(), digest.hex())
}

fn blob_chunk_id(name: &RepositoryName, session_id: &str) -> String {
    format!("{}/blob_chunk/{}", name.raw(), session_id)
}

fn read_file(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match fs::exists(&path) {
        Ok(true) => Ok(Some(fs::read(&path).map_err(|e| e.to_string())?)),
        Ok(false) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn write_file(path: &Path, data: &Vec<u8>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    }

    fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}
