use std::{fmt, str::FromStr};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum DocumentId {
    Rfc(u32),
    Draft(String),
}

impl DocumentId {
    pub fn canonical(&self) -> String {
        match self {
            Self::Rfc(number) => format!("RFC{number}"),
            Self::Draft(name) => name.clone(),
        }
    }

    pub fn cache_stem(&self) -> String {
        self.canonical().to_ascii_lowercase()
    }

    pub fn text_url(&self) -> String {
        match self {
            Self::Rfc(number) => format!("https://www.rfc-editor.org/rfc/rfc{number}.txt"),
            Self::Draft(name) => format!("https://www.ietf.org/archive/id/{name}.txt"),
        }
    }

    pub fn metadata_url(&self) -> Option<String> {
        match self {
            Self::Rfc(number) => Some(format!("https://www.rfc-editor.org/rfc/rfc{number}.json")),
            Self::Draft(_) => None,
        }
    }
}

impl FromStr for DocumentId {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let value = input.trim();
        if value.is_empty() {
            return Err(Error::InvalidIdentifier(input.to_owned()));
        }

        if value.len() >= 6 && value[..6].eq_ignore_ascii_case("draft-") {
            let normalized = value.to_ascii_lowercase();
            if normalized
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
                && !normalized.ends_with('-')
            {
                return Ok(Self::Draft(normalized));
            }
            return Err(Error::InvalidIdentifier(input.to_owned()));
        }

        let number = value
            .strip_prefix("RFC")
            .or_else(|| value.strip_prefix("rfc"))
            .unwrap_or(value)
            .parse::<u32>()
            .map_err(|_| Error::InvalidIdentifier(input.to_owned()))?;

        if number == 0 {
            return Err(Error::InvalidIdentifier(input.to_owned()));
        }
        Ok(Self::Rfc(number))
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.canonical())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rfc_forms() {
        assert_eq!("9110".parse::<DocumentId>().unwrap(), DocumentId::Rfc(9110));
        assert_eq!(
            "rfc9110".parse::<DocumentId>().unwrap(),
            DocumentId::Rfc(9110)
        );
        assert_eq!(
            "RFC9110".parse::<DocumentId>().unwrap(),
            DocumentId::Rfc(9110)
        );
    }

    #[test]
    fn parses_draft() {
        assert_eq!(
            "draft-ietf-httpbis-semantics-19"
                .parse::<DocumentId>()
                .unwrap(),
            DocumentId::Draft("draft-ietf-httpbis-semantics-19".to_owned())
        );
    }

    #[test]
    fn rejects_unsafe_or_empty_identifiers() {
        assert!("../9110".parse::<DocumentId>().is_err());
        assert!("draft-foo/bar".parse::<DocumentId>().is_err());
        assert!("0".parse::<DocumentId>().is_err());
    }
}
