use crate::registry_error::RegistryError;
use lazy_static::lazy_static;
use regex::Regex;

const REPO_NAME_REGEX: &str =
    "[a-z0-9]+((\\.|_|__|-+)[a-z0-9]+)*(\\/[a-z0-9]+((\\.|_|__|-+)[a-z0-9]+)*)*";

lazy_static! {
    static ref repo_name_regex: Regex = Regex::new(REPO_NAME_REGEX).unwrap();
}

#[derive(Debug)]
pub struct RepositoryName(String);

impl RepositoryName {
    pub fn new(name: &str, hostname: &str, port: Option<u16>) -> Result<Self, RegistryError> {
        let total_length =
        //  1 for '/'                                                            1 for ':'
            1 + hostname.len() + name.len() + port.map(|p| p.to_string().len() + 1).unwrap_or(0);

        if total_length > 255 {
            return Err(RegistryError::RepositoryNameInvalid(format!(
                "Repository name size exceeded. 'hostname:port/name' > 255 bytes'."
            )));
        }

        if repo_name_regex.is_match(name) {
            Ok(RepositoryName(name.to_string()))
        } else {
            Err(RegistryError::RepositoryNameInvalid(format!(
                "A repository name must match the following regular expression '{}'.",
                REPO_NAME_REGEX,
            )))
        }
    }

    pub fn raw(&self) -> String {
        self.0.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_name() {
        let raw_input = "coolest-image-name-ever:latest";
        let name = RepositoryName::new(raw_input, "localhost", Some(8080));
        assert!(name.is_ok());
        assert!(name.unwrap().raw() == raw_input);
    }

    #[test]
    fn long_name_is_invalid() {
        let long = r#"
awonderfulserenityhastakenpossessionofmyentiresoullikethesesweetmorningsofpringwhichIenjoywithmywholeheartImaloneandfeelthecharmofexistenceinthisspotwhichwascreatedforheblissofsoulslikemineIamsohappymydearfriendsoabsorbedintheexquisithahdhfsdhfhasdfhasdhfsadhfhasdfhasdfhasdhfasdhfshadfhasdfhasdfhasdhfasdhfmyhaahfriend"#;

        let name = RepositoryName::new(long, "localhost", Some(8080));
        assert!(name.is_err());
    }
}
