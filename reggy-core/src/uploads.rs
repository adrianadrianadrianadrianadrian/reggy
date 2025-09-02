use crate::{registry_error::RegistryError, repository_name::RepositoryName};
use uuid::Uuid;

pub async fn upload(name: RepositoryName, session_id: Uuid) -> Result<(), RegistryError> {
    Ok(())
}
