# MCP server

Start the Model Context Protocol server over stdio:

```console
rfc mcp
```

Configure an MCP client to launch that exact command. The process communicates only through stdio and does not open a network listener.

## Tools

### `get_rfc`

Retrieves authoritative plain-text RFC or Internet-Draft content.

Arguments:

- `identifier`: RFC number, prefixed RFC identifier, or complete Internet-Draft identifier;
- `offset_chars`: optional character offset, default `0`;
- `max_chars`: optional page length, default `50000`, maximum `200000`;
- `offline`: optional strict cache-only mode;
- `refresh`: optional forced refresh.

### `get_rfc_metadata`

Retrieves RFC Editor metadata and relationships for a published RFC.

Arguments:

- `identifier`: RFC number or prefixed RFC identifier;
- `offline`: optional strict cache-only mode;
- `refresh`: optional forced refresh.

Both tools publish JSON Schemas and return structured content.

## Example client configuration

The exact configuration key varies by client, but the command definition is typically equivalent to:

```json
{
  "mcpServers": {
    "rfc": {
      "command": "rfc",
      "args": ["mcp"]
    }
  }
}
```
