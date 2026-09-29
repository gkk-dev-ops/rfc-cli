pub mod cache;
pub mod client;
pub mod error;
pub mod identifier;
pub mod mcp;
pub mod model;

pub use client::{ClientOptions, RfcClient};
pub use error::{Error, Result};
pub use identifier::DocumentId;
pub use model::{CacheSource, DocumentResponse, MetadataResponse, RfcMetadata};
