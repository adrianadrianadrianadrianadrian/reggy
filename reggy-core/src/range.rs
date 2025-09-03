use crate::registry_error::RegistryError;
use lazy_static::lazy_static;
use regex::Regex;

const RANGE_REGEX: &str = "^[0-9]+-[0-9]+$";

lazy_static! {
    static ref range_regex: Regex = Regex::new(RANGE_REGEX).unwrap();
}

#[derive(Clone, Debug)]
pub struct Range {
    start: usize,
    end: usize,
}

impl Range {
    pub fn parse(input: &str) -> Result<Self, RegistryError> {
        if range_regex.is_match(input) {
            match input.split("-").collect::<Vec<_>>().as_slice() {
                [start, end] => {
                    let start =
                        usize::from_str_radix(start, 10).map_err(|_| RegistryError::SizeInvalid)?;
                    let end =
                        usize::from_str_radix(end, 10).map_err(|_| RegistryError::SizeInvalid)?;

                    if start >= end {
                        return Err(RegistryError::SizeInvalid);
                    }

                    Ok(Range { start, end })
                }
                _ => Err(RegistryError::SizeInvalid),
            }
        } else {
            // TODO: review errors
            Err(RegistryError::SizeInvalid)
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}
