# Agent integration

The CLI exposes a stable automation contract without embedding an LLM or requiring an API key.

## Recommended invocation

Use JSON for a single request:

```console
rfc --json info 9110
rfc --json show 9110
```

Agents should inspect both the process exit code and the response envelope. Successful structured responses include a schema version, canonical identifier, and authoritative source URL.

## Output rules

- stdout contains only the requested human or machine-readable result;
- stderr contains diagnostics or a structured error envelope;
- JSON is one complete response object;
- JSONL is line-oriented;
- MCP protocol messages are never mixed with application logs;
- no pager or interactive prompt is opened implicitly.

## Large documents

MCP content calls support character offsets and maximum lengths. The default page is 50,000 characters and a truncated response includes `next_offset_chars`. Clients should pass that value as `offset_chars` until no next offset is returned.

The maximum page size is 200,000 characters.

## Reliability

RFC text is immutable and cached indefinitely. Metadata is refreshed after 24 hours. If a refresh fails but cached data exists, the response can use it and marks the result as `stale_cache`.
