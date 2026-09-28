use std::{path::PathBuf, time::Duration};

use directories::ProjectDirs;

use crate::{
    cache::Cache,
    error::{Error, Result},
    identifier::DocumentId,
    model::{CacheSource, DocumentResponse, MetadataResponse, RawRfcMetadata, RfcMetadata},
};

const METADATA_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const USER_AGENT: &str = concat!("rfc-agent-cli/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub cache_dir: Option<PathBuf>,
    pub offline: bool,
    pub refresh: bool,
    pub timeout: Duration,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            cache_dir: None,
            offline: false,
            refresh: false,
            timeout: Duration::from_secs(20),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RfcClient {
    http: reqwest::Client,
    cache: Cache,
    offline: bool,
    refresh: bool,
}

impl RfcClient {
    pub fn new(options: ClientOptions) -> Result<Self> {
        let cache_dir = match options.cache_dir {
            Some(path) => path,
            None => ProjectDirs::from("dev", "gkk", "rfc-agent-cli")
                .map(|directories| directories.cache_dir().to_path_buf())
                .ok_or(Error::CacheDirectoryUnavailable)?,
        };
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(options.timeout)
            .build()?;
        Ok(Self {
            http,
            cache: Cache::new(cache_dir),
            offline: options.offline,
            refresh: options.refresh,
        })
    }

    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    pub async fn document(&self, id: &DocumentId) -> Result<DocumentResponse> {
        let url = id.text_url();
        let key = format!("documents/{}.txt", id.cache_stem());
        let (content, cache_source) = self.fetch(&key, &url, None).await?;
        Ok(DocumentResponse {
            schema_version: "1",
            kind: "rfc_document",
            identifier: id.canonical(),
            source_url: url,
            cache_source,
            total_chars: content.chars().count(),
            content,
            offset_chars: 0,
            next_offset_chars: None,
            truncated: false,
        })
    }

    pub async fn metadata(&self, id: &DocumentId) -> Result<MetadataResponse> {
        let url = id.metadata_url().ok_or(Error::DraftMetadataUnsupported)?;
        let key = format!("metadata/{}.json", id.cache_stem());
        let (content, cache_source) = self.fetch(&key, &url, Some(METADATA_MAX_AGE)).await?;
        let raw: RawRfcMetadata = serde_json::from_str(&content)?;
        Ok(MetadataResponse {
            schema_version: "1",
            kind: "rfc_metadata",
            identifier: id.canonical(),
            source_url: url,
            cache_source,
            metadata: RfcMetadata::from(raw),
        })
    }

    async fn fetch(
        &self,
        key: &str,
        url: &str,
        max_age: Option<Duration>,
    ) -> Result<(String, CacheSource)> {
        let cached = self.cache.read(key).await?;
        let fresh = match max_age {
            Some(max_age) => self.cache.is_fresh(key, max_age).await?,
            None => cached.is_some(),
        };

        if !self.refresh && fresh {
            return Ok((
                cached.expect("fresh cache entry must exist"),
                CacheSource::Cache,
            ));
        }

        if self.offline {
            return cached
                .map(|content| (content, CacheSource::StaleCache))
                .ok_or_else(|| Error::OfflineCacheMiss(key.to_owned()));
        }

        let response = match self.http.get(url).send().await {
            Ok(response) => response,
            Err(error) => {
                if let Some(content) = cached {
                    return Ok((content, CacheSource::StaleCache));
                }
                return Err(Error::Network(error));
            }
        };

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::NotFound(url.to_owned()));
        }
        let response = response.error_for_status()?;
        let content = response.text().await?;
        self.cache.write(key, &content).await?;
        Ok((content, CacheSource::Network))
    }
}
