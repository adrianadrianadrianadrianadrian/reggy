use lazy_static::lazy_static;
use regex::Regex;

const HASH_ALGORITHM_REGEX: &str = "[A-Fa-f0-9_+.-]+";
const HEX_REGEX: &str = "[A-Fa-f0-9]+";

lazy_static! {
    static ref hex_regex: Regex = Regex::new(HEX_REGEX).unwrap();
    static ref hash_algorithm_regex: Regex = Regex::new(HASH_ALGORITHM_REGEX).unwrap();
}

#[derive(Debug)]
pub struct Hex(String);

impl Hex {
    pub fn new(input: &str) -> Result<Self, String> {
        if input.is_empty() {
            return Err("A hex cannot be empty".to_string());
        }

        if hex_regex.is_match(input) {
            Ok(Hex(input.to_string()))
        } else {
            Err(format!(
                "A hex must match the following regular expression '{}'.",
                HEX_REGEX
            ))
        }
    }
}

#[derive(Debug)]
pub enum HashAlgorithm {
    SHA256,
}

impl HashAlgorithm {
    pub fn new(input: &str) -> Result<Self, String> {
        if input.is_empty() {
            return Err("A hash algorithm cannot be empty".to_string());
        }

        if hash_algorithm_regex.is_match(input) {
            match input {
                "sha256" | "SHA256" => Ok(HashAlgorithm::SHA256),
                _ => Err(format!(
                    "The hash algorithm '{}' is not currently supported.",
                    input
                )),
            }
        } else {
            Err(format!(
                "A hash algorithm must match the following regular expression '{}'.",
                HASH_ALGORITHM_REGEX
            ))
        }
    }
}

#[derive(Debug)]
pub struct Digest {
    algorithm: HashAlgorithm,
    hex: Hex,
}

impl Digest {
    pub fn new(input: &str) -> Result<Self, String> {
        if input.is_empty() {
            return Err("A digest cannot be empty".to_string());
        }

        match input.split(":").collect::<Vec<_>>().as_slice() {
            [algorithm, hex] => Ok(Self {
                algorithm: HashAlgorithm::new(algorithm)?,
                hex: Hex::new(hex)?,
            }),
            _ => {
                Err("A digest should be in the following format 'algorithm \":\" hex'".to_string())
            }
        }
    }

    pub fn hex(&self) -> String {
        self.hex.0.clone()
    }
}
