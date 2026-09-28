# rfc-agent-cli

`rfc-agent-cli` is a fast command-line interface and MCP server for authoritative IETF RFCs and Internet-Drafts. Every installation method exposes the same `rfc` executable backed by one Rust implementation.

It is designed for both people and software agents:

- readable terminal output by default;
- versioned JSON and JSONL output for automation;
- stable exit codes and structured errors;
- deterministic pagination for large documents;
- persistent caching and a strict offline mode;
- an MCP server over stdio;
- RFC content and metadata from official RFC Editor and IETF endpoints.

```console
rfc show 9110
rfc --json info 9110
rfc mcp
```

The project is a Rust rewrite of [wasi-master/rfc-cli](https://github.com/wasi-master/rfc-cli) and retains its attribution and MIT license.
