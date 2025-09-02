#[derive(Debug)]
pub enum RegistryError {
    BlobUnknown,
    BlobUploadInvalid(String),
    BlobUploadUnknown,
    DigestInvalid,
    ManifestBlobUnknown,
    ManifestInvalid,
    ManifestUnknown,
    ManifestUnverified,
    RepositoryNameInvalid(String),
    RepositoryNameUnknown,
    SizeInvalid,
    TagInvalid(String),
    Unauthorised,
    Denied,
    Unsupported,
    ReferenceInvalid(String)
}

impl RegistryError {
    pub fn as_string(&self) -> String {
        match self {
            RegistryError::BlobUnknown => "BLOB_UNKNOWN",
            RegistryError::BlobUploadInvalid(_) => "BLOB_UPLOAD_INVALID",
            RegistryError::BlobUploadUnknown => "BLOB_UPLOAD_UNKNOWN",
            RegistryError::DigestInvalid => "DIGEST_INVALID",
            RegistryError::ManifestBlobUnknown => "MANIFEST_BLOB_UNKNOWN",
            RegistryError::ManifestInvalid => "MANIFEST_INVALID",
            RegistryError::ManifestUnknown => "MANIFEST_UNKNOWN",
            RegistryError::ManifestUnverified => "MANIFEST_UNVERIFIED",
            RegistryError::RepositoryNameInvalid(_) => "NAME_INVALID",
            RegistryError::RepositoryNameUnknown => "NAME_UNKNOWN",
            RegistryError::SizeInvalid => "SIZE_INVALID",
            RegistryError::TagInvalid(_) => "TAG_INVALID",
            RegistryError::Unauthorised => "UNAUTHORIZED",
            RegistryError::Denied => "DENIED",
            RegistryError::Unsupported => "UNSUPPORTED",
            RegistryError::ReferenceInvalid(_) => ""
        }
        .to_string()
    }
}
