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

    pub fn into_string(&self) -> String {
        match self {
            Reference::Tag(tag) => tag.raw(),
            Reference::Digest(digest) => digest.hex()
        }
    }
}
