use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid RFC or Internet-Draft identifier: {0}")]
    InvalidIdentifier(String),

    #[error("document not found: {0}")]
    NotFound(String),

    #[error("metadata is currently available only for published RFCs")]
    DraftMetadataUnsupported,

    #[error("offline cache miss for {0}")]
    OfflineCacheMiss(String),

    #[error("request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("failed to access cache file {path}: {source}")]
    CacheIo {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("invalid metadata returned by the RFC Editor: {0}")]
    InvalidMetadata(#[from] serde_json::Error),

    #[error("unable to determine the platform cache directory")]
    CacheDirectoryUnavailable,

    #[error("MCP server failed: {0}")]
    Mcp(String),
}

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidIdentifier(_) | Self::DraftMetadataUnsupported => "INVALID_INPUT",
            Self::NotFound(_) => "NOT_FOUND",
            Self::OfflineCacheMiss(_) => "OFFLINE_CACHE_MISS",
            Self::Network(_) => "NETWORK_ERROR",
            Self::CacheIo { .. } | Self::CacheDirectoryUnavailable => "CACHE_ERROR",
            Self::InvalidMetadata(_) | Self::Mcp(_) => "INTERNAL_ERROR",
        }
    }

    pub fn exit_code(&self) -> u8 {
        match self {
            Self::InvalidIdentifier(_) | Self::DraftMetadataUnsupported => 2,
            Self::NotFound(_) => 4,
            Self::Network(_) => 5,
            Self::OfflineCacheMiss(_) => 6,
            Self::CacheIo { .. } | Self::CacheDirectoryUnavailable => 7,
            Self::InvalidMetadata(_) | Self::Mcp(_) => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
