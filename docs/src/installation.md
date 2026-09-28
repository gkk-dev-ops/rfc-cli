# Installation

The registry package is named `rfc-agent-cli`; the installed command is always `rfc`.

## Cargo

```console
cargo install rfc-agent-cli --locked
```

## Python / PyPI

For an isolated CLI installation, `pipx` is recommended:

```console
pipx install rfc-agent-cli
```

Plain `pip` also works:

```console
python -m pip install rfc-agent-cli
```

## npm

```console
npm install --global rfc-agent-cli
```

The npm package selects the correct prebuilt Rust binary for the current platform. It does not maintain a separate JavaScript implementation.

## Go

```console
go install github.com/gkk-dev-ops/rfc-cli/cmd/rfc@latest
```

The Go bootstrap downloads the matching official GitHub release archive, verifies its SHA-256 checksum, caches the binary, and delegates execution to it.
It resolves the Rust archive from the same version tag used by `go install`, including prerelease tags.

## From a checkout

```console
cargo install --path . --locked
```

Rust 1.98 or newer is required when compiling from source.
