use crate::{digest::Digest, range::Range};
use std::collections::HashMap;

pub struct Headers(HashMap<String, String>);

impl Headers {
    pub fn new(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    pub fn insert_docker_content_digest(&mut self, digest: &Digest) {
        self.0
            .insert("Docker-Content-Digest".to_string(), digest.hex());
    }

    pub fn insert_content_length(&mut self, length: usize) {
        self.0
            .insert("Content-Length".to_string(), length.to_string());
    }

    pub fn insert_location(&mut self, location: String) {
        self.0.insert("Location".to_string(), location);
    }

    pub fn insert_docker_upload_uuid(&mut self, uuid: &str) {
        self.0
            .insert("Docker-Upload-Uuid".to_string(), uuid.to_string());
    }

    pub fn insert_minimum_chunk_length(&mut self, min: usize) {
        self.0
            .insert("OCI-Chunk-Min-Length".to_string(), min.to_string());
    }

    pub fn insert_range(&mut self, start: usize, end: usize) {
        self.0
            .insert("Range".to_string(), format!("{}-{}", start, end));
    }
}

impl IntoIterator for Headers {
    type Item = (String, String);
    type IntoIter = <HashMap<String, String> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
