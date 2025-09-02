use crate::registry_error::RegistryError;
use lazy_static::lazy_static;
use regex::Regex;

const TAG_REGEX: &str = "[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}";

lazy_static! {
    static ref tag_regex: Regex = Regex::new(TAG_REGEX).unwrap();
}

#[derive(Debug)]
pub struct Tag(String);

impl Tag {
    pub fn new(input: &str) -> Result<Self, RegistryError> {
        if tag_regex.is_match(input) {
            Ok(Tag(input.to_string()))
        } else {
            Err(RegistryError::TagInvalid(format!(
                "A tag must match the following regular expression '{}'.",
                TAG_REGEX,
            )))
        }
    }
}
