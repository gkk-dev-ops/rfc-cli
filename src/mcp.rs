use std::str::FromStr;

use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::wrapper::{Json, Parameters},
    model::{Implementation, ServerCapabilities, ServerConfig},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{DocumentId, DocumentResponse, Error, MetadataResponse, RfcClient};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentRequest {
    /// RFC number (`9110` or `RFC9110`) or full Internet-Draft name.
    pub identifier: String,
    /// Character offset at which to start the response. Defaults to zero.
    #[serde(default)]
    pub offset_chars: usize,
    /// Maximum characters to return. Defaults to 50,000 and is capped at 200,000.
    pub max_chars: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct RfcMcpServer {
    client: RfcClient,
}

impl RfcMcpServer {
    pub fn new(client: RfcClient) -> Self {
        Self { client }
    }

    fn invalid(error: impl ToString) -> McpError {
        McpError::invalid_params(error.to_string(), None)
    }

    fn internal(error: impl ToString) -> McpError {
        McpError::internal_error(error.to_string(), None)
    }

    fn client_error(error: Error) -> McpError {
        match error.code() {
            "INVALID_INPUT" => Self::invalid(error),
            _ => Self::internal(error),
        }
    }
}

#[tool_router]
impl RfcMcpServer {
    #[tool(
        name = "get_rfc",
        description = "Retrieve a bounded page of the authoritative plain-text form of an RFC or Internet-Draft. Returns structured content with a canonical identifier, source URL, total size, and next offset."
    )]
    async fn get_rfc(
        &self,
        Parameters(request): Parameters<DocumentRequest>,
    ) -> Result<Json<DocumentResponse>, McpError> {
        let id = DocumentId::from_str(&request.identifier).map_err(Self::invalid)?;
        let max_chars = request.max_chars.unwrap_or(50_000);
        if max_chars == 0 || max_chars > 200_000 {
            return Err(Self::invalid("max_chars must be between 1 and 200000"));
        }
        self.client
            .document(&id)
            .await
            .map(|response| response.page(request.offset_chars, max_chars))
            .map(Json)
            .map_err(Self::client_error)
    }

    #[tool(
        name = "get_rfc_metadata",
        description = "Retrieve structured RFC Editor metadata, including title, authors, status, relationships, DOI, and errata URL."
    )]
    async fn get_rfc_metadata(
        &self,
        Parameters(request): Parameters<DocumentRequest>,
    ) -> Result<Json<MetadataResponse>, McpError> {
        let id = DocumentId::from_str(&request.identifier).map_err(Self::invalid)?;
        self.client
            .metadata(&id)
            .await
            .map(Json)
            .map_err(Self::client_error)
    }
}

#[tool_handler]
impl ServerHandler for RfcMcpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "rfc-agent-cli",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Use get_rfc_metadata to discover status and relationships. Use get_rfc only when authoritative document text is needed; responses include canonical source URLs.",
            )
    }
}

pub async fn serve(client: RfcClient) -> crate::Result<()> {
    RfcMcpServer::new(client)
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|error| crate::Error::Mcp(error.to_string()))?
        .waiting()
        .await
        .map_err(|error| crate::Error::Mcp(error.to_string()))?;
    Ok(())
}
