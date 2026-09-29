use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CacheSource {
    Network,
    Cache,
    StaleCache,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentResponse {
    pub schema_version: &'static str,
    pub kind: &'static str,
    pub identifier: String,
    pub source_url: String,
    pub cache_source: CacheSource,
    pub content: String,
    pub offset_chars: usize,
    pub total_chars: usize,
    pub next_offset_chars: Option<usize>,
    pub truncated: bool,
}

impl DocumentResponse {
    pub fn page(mut self, offset_chars: usize, max_chars: usize) -> Self {
        let characters: Vec<char> = self.content.chars().collect();
        let total_chars = characters.len();
        let start = offset_chars.min(total_chars);
        let end = start.saturating_add(max_chars).min(total_chars);
        self.content = characters[start..end].iter().collect();
        self.offset_chars = start;
        self.total_chars = total_chars;
        self.next_offset_chars = (end < total_chars).then_some(end);
        self.truncated = end < total_chars;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetadataResponse {
    pub schema_version: &'static str,
    pub kind: &'static str,
    pub identifier: String,
    pub source_url: String,
    pub cache_source: CacheSource,
    pub metadata: RfcMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RfcMetadata {
    pub doc_id: String,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub format: Vec<String>,
    pub page_count: Option<String>,
    pub pub_status: Option<String>,
    pub status: Option<String>,
    pub source: Option<String>,
    pub abstract_text: Option<String>,
    pub pub_date: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub obsoletes: Vec<String>,
    #[serde(default)]
    pub obsoleted_by: Vec<String>,
    #[serde(default)]
    pub updates: Vec<String>,
    #[serde(default)]
    pub updated_by: Vec<String>,
    #[serde(default)]
    pub see_also: Vec<String>,
    pub doi: Option<String>,
    pub errata_url: Option<String>,
    pub draft: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawRfcMetadata {
    pub doc_id: String,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub format: Vec<String>,
    pub page_count: Option<String>,
    pub pub_status: Option<String>,
    pub status: Option<String>,
    pub source: Option<String>,
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    pub pub_date: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub obsoletes: Vec<String>,
    #[serde(default)]
    pub obsoleted_by: Vec<String>,
    #[serde(default)]
    pub updates: Vec<String>,
    #[serde(default)]
    pub updated_by: Vec<String>,
    #[serde(default)]
    pub see_also: Vec<String>,
    pub doi: Option<String>,
    pub errata_url: Option<String>,
    pub draft: Option<String>,
}

impl From<RawRfcMetadata> for RfcMetadata {
    fn from(raw: RawRfcMetadata) -> Self {
        Self {
            doc_id: raw.doc_id,
            title: raw.title,
            authors: raw.authors,
            format: raw.format,
            page_count: raw.page_count,
            pub_status: raw.pub_status,
            status: raw.status,
            source: raw.source,
            abstract_text: raw.abstract_text,
            pub_date: raw.pub_date,
            keywords: raw.keywords,
            obsoletes: raw.obsoletes,
            obsoleted_by: raw.obsoleted_by,
            updates: raw.updates,
            updated_by: raw.updated_by,
            see_also: raw.see_also,
            doi: raw.doi,
            errata_url: raw.errata_url,
            draft: raw.draft,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(content: &str) -> DocumentResponse {
        DocumentResponse {
            schema_version: "1",
            kind: "rfc_document",
            identifier: "RFC1".to_owned(),
            source_url: "https://www.rfc-editor.org/rfc/rfc1.txt".to_owned(),
            cache_source: CacheSource::Cache,
            content: content.to_owned(),
            offset_chars: 0,
            total_chars: content.chars().count(),
            next_offset_chars: None,
            truncated: false,
        }
    }

    #[test]
    fn pages_by_unicode_characters_without_splitting_text() {
        let page = response("aąbc").page(1, 2);
        assert_eq!(page.content, "ąb");
        assert_eq!(page.offset_chars, 1);
        assert_eq!(page.total_chars, 4);
        assert_eq!(page.next_offset_chars, Some(3));
        assert!(page.truncated);
    }

    #[test]
    fn returns_an_empty_terminal_page_for_large_offset() {
        let page = response("abc").page(100, 10);
        assert_eq!(page.content, "");
        assert_eq!(page.offset_chars, 3);
        assert_eq!(page.next_offset_chars, None);
        assert!(!page.truncated);
    }
}
