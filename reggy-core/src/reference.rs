use crate::{digest::Digest, registry_error::RegistryError, tag::Tag};

#[derive(Debug)]
pub enum Reference {
    Tag(Tag),
    Digest(Digest),
}

impl Reference {
    pub fn new(reference: &str) -> Result<Self, RegistryError> {
        if let Ok(digest) = Digest::new(reference) {
            return Ok(Reference::Digest(digest));
        }
        
        if let Ok(tag) = Tag::new(reference) {
            return Ok(Reference::Tag(tag));
        }
        
        Err(RegistryError::ReferenceInvalid("A reference must be either a digest or tag.".to_string()))
    }

    pub fn into_string(&self) -> String {
        match self {
            Reference::Tag(tag) => tag.raw(),
            Reference::Digest(digest) => digest.hex()
        }
    }
}
