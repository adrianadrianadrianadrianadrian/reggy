use crate::{digest::Digest, registry_error::RegistryError, tag::Tag};

#[derive(Debug)]
pub enum Reference {
    Tag(Tag),
    Digest(Digest),
}

impl Reference {
    pub fn new(reference: &str) -> Result<Self, RegistryError> {
        todo!()
    }
}
