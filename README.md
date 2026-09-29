# rfc-cli

An agent-friendly command-line interface and MCP server for authoritative IETF RFCs and Internet-Drafts.

This project is a Rust rewrite of [wasi-master/rfc-cli](https://github.com/wasi-master/rfc-cli). It preserves the concise `rfc` command while adding deterministic machine output, explicit errors, persistent caching, offline operation, and MCP tools.

> **Status:** `0.2.0-alpha.1`. The Rust CLI works; registry packages and release automation are being prepared and have not been published.

Full documentation is available at [gkk-dev-ops.github.io/rfc-cli](https://gkk-dev-ops.github.io/rfc-cli/).

## Usage

```console
rfc show 9110
rfc show RFC9110
rfc show draft-ietf-httpbis-semantics-19
rfc info 9110

rfc --json info 9110
rfc --output json show 9110

rfc cache status
rfc --offline show 9110
rfc --refresh info 9110

rfc mcp
```

`rfc show <identifier>` remains compatible with the original Python CLI. The hidden `--pager` option is accepted for script compatibility, but the new CLI never opens an implicit pager. Callers remain in control of stdout.

## Agent contract

- Human-readable output is the default.
- `--json` and `--output json` emit one versioned JSON object to stdout.
- Diagnostics and JSON error envelopes are written to stderr.
- MCP is served over stdio and never mixes logs with protocol messages.
- Every successful structured response includes a canonical identifier and authoritative source URL.
- MCP document responses default to 50,000 characters and expose `next_offset_chars` for deterministic pagination.
- RFC text is immutable and cached indefinitely. RFC metadata is refreshed after 24 hours.
- `--offline` never performs a network request and returns exit code `6` on a cache miss.
- A stale cache entry is used when the network is unavailable and is marked `stale_cache`.

### Exit codes

| Code | Meaning |
| ---: | --- |
| `0` | Success |
| `1` | Internal or MCP error |
| `2` | Invalid input |
| `4` | RFC or draft not found |
| `5` | Network error |
| `6` | Offline cache miss |
| `7` | Cache error |

### MCP tools

| Tool | Purpose |
| --- | --- |
| `get_rfc` | Retrieve authoritative plain-text RFC or Internet-Draft content |
| `get_rfc_metadata` | Retrieve RFC Editor metadata, relationships, DOI, and errata URL |

Both tools return structured content with a generated JSON Schema.

## Installation

The executable is always named `rfc`. The package name is `rfc-agent-cli` on Cargo, PyPI, and npm.

### Cargo

```console
cargo install --path . --locked
# After publication: cargo install rfc-agent-cli --locked
```

### PyPI / pipx

The project uses Maturin's binary packaging mode:

```console
maturin build --release --bindings bin
pipx install dist/rfc_agent_cli-*.whl
# After publication: pipx install rfc-agent-cli
```

### npm

Release automation generates a package-manager shim and attaches the matching prebuilt Rust binary through platform-specific optional dependencies, without downloading or executing code in a postinstall script.

```console
npm install --global rfc-agent-cli
```

### Go

Go cannot compile the Rust implementation directly. `cmd/rfc` is a small bootstrap command that downloads the official release artifact, verifies its SHA-256 checksum, caches it, and delegates execution to it.

```console
go install github.com/gkk-dev-ops/rfc-cli/cmd/rfc@latest
```

All installation methods expose the same Rust executable and output contract. There are no independent protocol implementations to drift apart.

## Development

Rust 1.98 or newer is required.

```console
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- info 9110
```

The implementation uses official RFC Editor text and per-RFC JSON metadata. Internet-Drafts use the IETF archive.

## Attribution

Originally forked from [wasi-master/rfc-cli](https://github.com/wasi-master/rfc-cli), created by Arian Mollik Wasi. The original copyright and MIT license are retained.
