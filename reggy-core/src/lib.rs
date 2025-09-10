use crate::headers::Headers;

pub mod blob;
pub mod digest;
pub mod headers;
pub mod manifest;
pub mod range;
pub mod reference;
pub mod registry_error;
pub mod repository_name;
pub mod tag;

pub type Response<T> = (T, Headers);
